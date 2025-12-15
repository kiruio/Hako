use crate::account::offline_uuid;
use crate::game::args::{Features, collect_game_args, collect_jvm_args};
use crate::game::classpath::build_classpath;
use crate::game::instance::GameInstance;
use crate::game::java::find_java;
use crate::game::natives::{extract_natives, get_natives_directory};
use crate::game::profile::load_version_profile;
use crate::task::error::{TaskError, TaskResult};
use crate::task::lock::LockKey;
use crate::task::main_task::{BlockingTask, TaskContext, TaskType};
use crate::task::sub_task::{SubTask, SubTaskChain, SubTaskContext};
use anyhow::{Context, Result, anyhow};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::io::AsyncBufReadExt;
use tokio::process::{Child, Command};

const MAX_WAIT_TIME: Duration = Duration::from_secs(30);

pub struct StartGameTask {
	pub instance: GameInstance,
	pub java_path: Option<PathBuf>,
	pub jvm_args: Vec<String>,
	pub game_args: Vec<String>,
}

impl TaskType for StartGameTask {
	const TYPE_NAME: &'static str = "start_game";
}

#[derive(Clone)]
struct StartShared {
	game_dir: PathBuf,
	version_id: String,
	profile: Option<crate::game::profile::VersionProfile>,
	features: Features,
	natives_dir: Option<PathBuf>,
	java_bin: Option<PathBuf>,
	classpath: Option<String>,
	username: String,
	uuid: String,
	jvm_args: Vec<String>,
	game_args: Vec<String>,
}

#[async_trait::async_trait]
impl BlockingTask for StartGameTask {
	type Output = ();

	fn locks(&self) -> Vec<LockKey> {
		vec![LockKey::global("start_game")]
	}

	fn queueable(&self) -> bool {
		false
	}

	async fn execute(&mut self, ctx: &TaskContext) -> TaskResult<Self::Output> {
		let game_dir = self.instance.cluster_path.clone();
		let version_id = self.instance.version.clone();

		tracing::info!(
			"start task begin: dir={}, version={}",
			game_dir.display(),
			version_id
		);

		let shared = StartShared {
			game_dir,
			version_id,
			profile: None,
			features: Features::default(),
			natives_dir: None,
			java_bin: None,
			classpath: None,
			username: "demo".to_string(),
			uuid: offline_uuid("demo").to_string(),
			jvm_args: self.jvm_args.clone(),
			game_args: self.game_args.clone(),
		};

		let shared = Arc::new(tokio::sync::Mutex::new(shared));

		let mut chain = SubTaskChain::new();
		chain.add(AccountPrepareTask {
			shared: shared.clone(),
		});
		chain.add(EnvPrepareTask {
			shared: shared.clone(),
			java_path: self.java_path.clone(),
		});
		chain.add(IntegrityCheckTask {
			shared: shared.clone(),
		});
		chain.add(LaunchAndWaitTask { shared });

		let sub_ctx = SubTaskContext::new(ctx.cancelled_receiver());
		chain
			.execute(&sub_ctx)
			.await
			.map_err(|e| TaskError::Failed(e.to_string()))
	}
}

struct AccountPrepareTask {
	shared: Arc<tokio::sync::Mutex<StartShared>>,
}

#[async_trait::async_trait]
impl SubTask for AccountPrepareTask {
	async fn execute(&self, _ctx: &SubTaskContext) -> Result<(), TaskError> {
		// 目前账号准备逻辑极简，保留为独立子任务以便将来扩展
		let mut shared = self.shared.lock().await;
		shared.username = "demo".to_string();
		shared.uuid = offline_uuid(&shared.username).to_string();
		Ok(())
	}
}

struct EnvPrepareTask {
	shared: Arc<tokio::sync::Mutex<StartShared>>,
	java_path: Option<PathBuf>,
}

#[async_trait::async_trait]
impl SubTask for EnvPrepareTask {
	async fn execute(&self, _ctx: &SubTaskContext) -> Result<(), TaskError> {
		let mut shared = self.shared.lock().await;

		let profile = load_version_profile(&shared.game_dir, &shared.version_id)
			.map_err(|e| TaskError::Failed(format!("load version profile failed: {e}")))?;

		let natives_dir = get_natives_directory(&shared.game_dir, &shared.version_id)
			.map_err(|e| TaskError::Failed(e.to_string()))?;
		extract_natives(&shared.game_dir, &profile, &natives_dir, &shared.features)
			.map_err(|e| TaskError::Failed(e.to_string()))?;

		let java_bin =
			find_java(self.java_path.clone()).map_err(|e| TaskError::Failed(e.to_string()))?;
		tracing::info!("java resolved: {}", java_bin.display());

		let cp = build_classpath(
			&shared.game_dir,
			&shared.version_id,
			&profile,
			&shared.features,
		)
		.map_err(|e| TaskError::Failed(e.to_string()))?;

		let assets_index = profile
			.assets
			.as_deref()
			.unwrap_or(&shared.version_id)
			.to_string();

		let jvm_args = collect_jvm_args(
			&profile,
			&shared.game_dir,
			&shared.version_id,
			&cp,
			&assets_index,
			&shared.username,
			&shared.uuid,
			&get_natives_directory(&shared.game_dir, &shared.version_id)
				.map_err(|e| TaskError::Failed(e.to_string()))?,
			&shared.features,
		);

		let game_args = collect_game_args(
			&shared.game_dir,
			&shared.version_id,
			&profile,
			&shared.username,
			&shared.uuid,
			&assets_index,
			&shared.features,
		);

		shared.profile = Some(profile);
		shared.natives_dir = Some(natives_dir);
		shared.java_bin = Some(java_bin);
		shared.classpath = Some(cp);
		shared.jvm_args.extend(jvm_args);
		shared.game_args.extend(game_args);

		Ok(())
	}
}

struct IntegrityCheckTask {
	shared: Arc<tokio::sync::Mutex<StartShared>>,
}

#[async_trait::async_trait]
impl SubTask for IntegrityCheckTask {
	async fn execute(&self, _ctx: &SubTaskContext) -> Result<(), TaskError> {
		let shared = self.shared.lock().await;

		if shared.profile.is_none()
			|| shared.java_bin.is_none()
			|| shared.classpath.is_none()
			|| shared.natives_dir.is_none()
		{
			return Err(TaskError::Failed("start context incomplete".into()));
		}

		if shared.jvm_args.is_empty() {
			return Err(TaskError::Failed("jvm args empty".into()));
		}

		if shared.game_args.is_empty() {
			return Err(TaskError::Failed("game args empty".into()));
		}

		Ok(())
	}
}

struct LaunchAndWaitTask {
	shared: Arc<tokio::sync::Mutex<StartShared>>,
}

#[async_trait::async_trait]
impl SubTask for LaunchAndWaitTask {
	async fn execute(&self, ctx: &SubTaskContext) -> Result<(), TaskError> {
		let shared = self.shared.lock().await;
		let profile = shared
			.profile
			.clone()
			.ok_or_else(|| TaskError::Failed("profile missing".into()))?;
		let java_bin = shared
			.java_bin
			.clone()
			.ok_or_else(|| TaskError::Failed("java missing".into()))?;
		let game_dir = shared.game_dir.clone();
		let jvm_args = shared.jvm_args.clone();
		let game_args = shared.game_args.clone();
		drop(shared);

		let mut cmd = Command::new(java_bin);

		#[cfg(windows)]
		{
			const CREATE_NEW_PROCESS_GROUP: u32 = 0x0000_0200;
			cmd.creation_flags(CREATE_NEW_PROCESS_GROUP);
		}

		cmd.args(jvm_args)
			.arg(
				profile
					.main_class
					.context("mainClass missing")
					.map_err(|e| TaskError::Failed(e.to_string()))?,
			)
			.args(game_args)
			.current_dir(&game_dir)
			.stdin(std::process::Stdio::null())
			.stdout(std::process::Stdio::piped())
			.stderr(std::process::Stdio::piped());

		let mut child = cmd
			.spawn()
			.context("Failed to start game process")
			.map_err(|e| TaskError::Failed(e.to_string()))?;
		tracing::info!("game process spawned, waiting for game to initialize");

		let mut cancelled = ctx.cancelled.clone();
		let start_time = Instant::now();

		let stdout = child
			.stdout
			.take()
			.context("Failed to get stdout")
			.map_err(|e| TaskError::Failed(e.to_string()))?;
		let stderr = child
			.stderr
			.take()
			.context("Failed to get stderr")
			.map_err(|e| TaskError::Failed(e.to_string()))?;

		let mut stdout_reader = tokio::io::BufReader::new(stdout).lines();
		let mut stderr_reader = tokio::io::BufReader::new(stderr).lines();

		loop {
			tokio::select! {
				line = stdout_reader.next_line() => {
					if let Some(_done) = handle_line(line, &mut child, start_time, "stdout")
						.map_err(|e| TaskError::Failed(e.to_string()))?
					{
						return Ok(());
					}
				}
				line = stderr_reader.next_line() => {
					if let Some(_done) = handle_line(line, &mut child, start_time, "stderr")
						.map_err(|e| TaskError::Failed(e.to_string()))?
					{
						return Ok(());
					}
				}
				_ = tokio::time::sleep(tokio::time::Duration::from_millis(100)) => {
					if let Some(status) = child.try_wait().map_err(|e| TaskError::Failed(e.to_string()))? {
						if !status.success() {
							return Err(TaskError::Failed(format!("Game exited with status: {:?}", status.code())));
						}
						tracing::info!("game process exited, task completed");
						return Ok(());
					}

					if start_time.elapsed() > MAX_WAIT_TIME {
						tracing::warn!("timeout waiting for game initialization, assuming it started");
						return Ok(());
					}
				}
				_ = cancelled.changed() => {
					let _ = child.kill().await;
					return Err(TaskError::Cancelled);
				}
			}
		}
	}
}

fn is_game_initialized(log: &str) -> bool {
	let lower = log.to_lowercase();
	lower.contains("lwjgl version")
		|| lower.contains("lwjgl openal")
		|| lower.contains("openal initialized")
		|| lower.contains("starting up soundsystem")
		|| lower.contains("setting user:")
}

fn handle_line(
	line: std::io::Result<Option<String>>,
	child: &mut Child,
	start_time: Instant,
	label: &str,
) -> Result<Option<()>> {
	match line {
		Ok(Some(log)) if is_game_initialized(&log) => {
			tracing::info!("game initialized (detected from {label}), task completed");
			Ok(Some(()))
		}
		Ok(Some(_)) => Ok(None),
		Ok(None) => {
			if let Some(status) = child.try_wait()? {
				if !status.success() {
					return Err(anyhow!("Game exited with status: {:?}", status.code()));
				}
				tracing::info!("game process exited ({label} closed), task completed");
				return Ok(Some(()));
			}

			if start_time.elapsed() > MAX_WAIT_TIME {
				tracing::warn!(
					"timeout waiting for game initialization ({label} closed), assuming it started"
				);
				return Ok(Some(()));
			}

			Ok(None)
		}
		Err(err) => {
			tracing::warn!("read {label} failed: {:?}", err);
			Ok(None)
		}
	}
}
