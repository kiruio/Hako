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
use anyhow::{Context, Result};
use std::path::PathBuf;
use tokio::io::AsyncBufReadExt;
use tokio::process::Command;

pub struct StartGameTask {
	pub instance: GameInstance,
	pub java_path: Option<PathBuf>,
	pub jvm_args: Vec<String>,
	pub game_args: Vec<String>,
}

impl TaskType for StartGameTask {
	const TYPE_NAME: &'static str = "start_game";
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
		run_start(ctx, self)
			.await
			.map_err(|e| TaskError::Failed(e.to_string()))
	}
}

async fn run_start(ctx: &TaskContext, task: &StartGameTask) -> Result<()> {
	let game_dir = &task.instance.cluster_path;
	let version_id = &task.instance.version;

	tracing::info!(
		"start task begin: dir={}, version={}",
		game_dir.display(),
		version_id
	);

	let profile = load_version_profile(game_dir, version_id)?;

	let features = Features::default();
	let natives_dir = get_natives_directory(game_dir, version_id)?;
	extract_natives(game_dir, &profile, &natives_dir, &features)?;

	let java_bin = find_java(task.java_path.clone())?;
	tracing::info!("java resolved: {}", java_bin.display());

	let cp = build_classpath(game_dir, version_id, &profile, &features)?;
	let assets_index = profile.assets.as_deref().unwrap_or(version_id);
	let username = "demo";
	let uuid = offline_uuid(username);

	let mut jvm_args = collect_jvm_args(
		&profile,
		game_dir,
		version_id,
		&cp,
		assets_index,
		username,
		&uuid.to_string(),
		&natives_dir,
		&features,
	);
	jvm_args.extend(task.jvm_args.clone());

	let mut game_args = collect_game_args(
		game_dir,
		version_id,
		&profile,
		username,
		&uuid.to_string(),
		assets_index,
		&features,
	);
	game_args.extend(task.game_args.clone());

	let mut cmd = Command::new(java_bin);

	#[cfg(windows)]
	{
		const CREATE_NEW_PROCESS_GROUP: u32 = 0x0000_0200;
		cmd.creation_flags(CREATE_NEW_PROCESS_GROUP);
	}

	cmd.args(jvm_args)
		.arg(profile.main_class.context("mainClass missing")?)
		.args(game_args)
		.current_dir(game_dir)
		.stdin(std::process::Stdio::null())
		.stdout(std::process::Stdio::piped())
		.stderr(std::process::Stdio::piped());

	let mut child = cmd.spawn().context("Failed to start game process")?;
	tracing::info!("game process spawned, waiting for game to initialize");

	let mut cancelled = ctx.cancelled.clone();
	let start_time = std::time::Instant::now();
	const MAX_WAIT_TIME: std::time::Duration = std::time::Duration::from_secs(30);

	let stdout = child.stdout.take().context("Failed to get stdout")?;
	let stderr = child.stderr.take().context("Failed to get stderr")?;

	let mut stdout_reader = tokio::io::BufReader::new(stdout).lines();
	let mut stderr_reader = tokio::io::BufReader::new(stderr).lines();

	loop {
		tokio::select! {
			line = stdout_reader.next_line() => {
				if let Ok(Some(log)) = line {
					if is_game_initialized(&log) {
						tracing::info!("game initialized (detected from log), task completed");
						return Ok(());
					}
				}
			}
			line = stderr_reader.next_line() => {
				if let Ok(Some(log)) = line {
					if is_game_initialized(&log) {
						tracing::info!("game initialized (detected from log), task completed");
						return Ok(());
					}
				}
			}
			_ = tokio::time::sleep(tokio::time::Duration::from_millis(100)) => {
				if let Ok(Some(status)) = child.try_wait() {
					if !status.success() {
						return Err(anyhow::anyhow!("Game exited with status: {:?}", status.code()));
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
			return Err(anyhow::anyhow!("Game start cancelled"));
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
