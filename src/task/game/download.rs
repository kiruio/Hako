use crate::game::args::{Features, current_arch, current_os_key, rule_allows};
use crate::game::profile::{Library, VersionProfile, load_version_profile};
use crate::net::download::{DownloadClient, DownloadRequest};
use crate::task::error::{TaskError, TaskResult};
use crate::task::lock::LockKey;
use crate::task::main_task::{ConcurrentTask, TaskContext, TaskType};
use crate::task::sub_task::{SubTask, SubTaskChain, SubTaskContext};
use anyhow::Context;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use tokio::fs;
use tokio::sync::watch;

pub struct DownloadGameTask {
	pub cluster_path: PathBuf,
	pub version: String,
	pub progress: Option<Arc<Mutex<DownloadProgressState>>>,
}

impl TaskType for DownloadGameTask {
	const TYPE_NAME: &'static str = "download_game";
}

#[async_trait::async_trait]
impl ConcurrentTask for DownloadGameTask {
	type Output = ();

	fn locks(&self) -> Vec<LockKey> {
		vec![LockKey::resource("download_game", self.version.as_str())]
	}

	fn max_concurrent(&self) -> Option<usize> {
		Some(2)
	}

	async fn execute(&mut self, ctx: &TaskContext) -> TaskResult<Self::Output> {
		let game_dir = self.cluster_path.clone();
		let version_id = self.version.clone();

		let client = Arc::new(DownloadClient::new().map_err(|e| TaskError::Failed(e.to_string()))?);
		let cancel = ctx.cancelled_receiver();

		let shared_profile: Arc<Mutex<Option<VersionProfile>>> = Arc::new(Mutex::new(None));

		let mut chain = SubTaskChain::new();
		chain.add(EnsureProfileTask {
			client: client.clone(),
			game_dir: game_dir.clone(),
			version_id: version_id.clone(),
			progress: self.progress.clone(),
			profile: shared_profile.clone(),
		});
		chain.add_parallel(
			vec![
				Arc::new(ClientJarTask {
					client: client.clone(),
					game_dir: game_dir.clone(),
					version_id: version_id.clone(),
					progress: self.progress.clone(),
					profile: shared_profile.clone(),
				}) as Arc<dyn SubTask>,
				Arc::new(CoreLibTask {
					client: client.clone(),
					game_dir: game_dir.clone(),
					version_id: version_id.clone(),
					progress: self.progress.clone(),
					profile: shared_profile.clone(),
				}) as Arc<dyn SubTask>,
				Arc::new(AssetsTask {
					client: client.clone(),
					game_dir: game_dir.clone(),
					version_id: version_id.clone(),
					progress: self.progress.clone(),
					profile: shared_profile.clone(),
				}) as Arc<dyn SubTask>,
			],
			None,
		);

		let sub_ctx = SubTaskContext::new(cancel);
		chain.execute(&sub_ctx).await?;

		set_progress(
			&self.progress,
			format!("{} 下载完成", version_id),
			0,
			None,
			0.0,
			true,
		);

		Ok(())
	}
}

struct EnsureProfileTask {
	client: Arc<DownloadClient>,
	game_dir: PathBuf,
	version_id: String,
	progress: Option<Arc<Mutex<DownloadProgressState>>>,
	profile: Arc<Mutex<Option<VersionProfile>>>,
}

#[async_trait::async_trait]
impl SubTask for EnsureProfileTask {
	async fn execute(&self, ctx: &SubTaskContext) -> Result<(), TaskError> {
		let mut cancel = ctx.cancelled.clone();
		if let Some(existing) = self.profile.lock().ok().and_then(|p| p.clone()) {
			let _ = existing;
			return Ok(());
		}

		let version_json = self
			.game_dir
			.join("versions")
			.join(&self.version_id)
			.join(format!("{}.json", self.version_id));

		if !version_json.exists() {
			tracing::info!("download version json for {}", self.version_id);
			set_progress(
				&self.progress,
				format!("下载版本元数据 {}", self.version_id),
				0,
				None,
				0.0,
				false,
			);
			let dir = version_json
				.parent()
				.ok_or_else(|| TaskError::Failed("invalid version path".into()))?;
			fs::create_dir_all(dir)
				.await
				.map_err(|e| TaskError::Failed(e.to_string()))?;

			let meta_url = resolve_version_url(&self.version_id).await?;
			tracing::debug!("manifest url for {}: {}", self.version_id, meta_url);
			self.client
				.download(
					DownloadRequest::new(meta_url, &version_json),
					|p| {
						set_progress(
							&self.progress,
							format!("下载版本元数据 {}", self.version_id),
							p.downloaded,
							p.total,
							p.speed_bps,
							false,
						)
					},
					Some(cancel.clone()),
				)
				.await
				.map_err(|e| TaskError::Failed(e.to_string()))?;
		}

		let profile = load_version_profile(&self.game_dir, &self.version_id)
			.map_err(|e| TaskError::Failed(e.to_string()))?;
		if let Ok(mut guard) = self.profile.lock() {
			*guard = Some(profile);
		}
		Ok(())
	}
}

fn read_profile(profile: &Arc<Mutex<Option<VersionProfile>>>) -> Option<VersionProfile> {
	profile.lock().ok().and_then(|p| p.clone())
}

struct ClientJarTask {
	client: Arc<DownloadClient>,
	game_dir: PathBuf,
	version_id: String,
	progress: Option<Arc<Mutex<DownloadProgressState>>>,
	profile: Arc<Mutex<Option<VersionProfile>>>,
}

#[async_trait::async_trait]
impl SubTask for ClientJarTask {
	async fn execute(&self, ctx: &SubTaskContext) -> Result<(), TaskError> {
		let mut cancel = ctx.cancelled.clone();
		let profile = read_profile(&self.profile)
			.ok_or_else(|| TaskError::Failed("profile missing for client jar".into()))?;
		if let Some(downloads) = profile.downloads.as_ref() {
			if let Some(client_dl) = downloads.client.as_ref() {
				if let Some(url) = &client_dl.url {
					let dest = self
						.game_dir
						.join("versions")
						.join(&self.version_id)
						.join(format!("{}.jar", self.version_id));
					if dest.exists() {
						return Ok(());
					}
					check_cancel(&mut cancel).await?;
					tracing::info!("download client {}", dest.display());
					self.client
						.download(
							DownloadRequest::new(url.clone(), dest),
							|p| {
								set_progress(
									&self.progress,
									format!("下载客户端 {}", self.version_id),
									p.downloaded,
									p.total,
									p.speed_bps,
									false,
								)
							},
							Some(cancel.clone()),
						)
						.await
						.map_err(|e| TaskError::Failed(e.to_string()))?;
				}
			}
		}
		Ok(())
	}
}

struct CoreLibTask {
	client: Arc<DownloadClient>,
	game_dir: PathBuf,
	version_id: String,
	progress: Option<Arc<Mutex<DownloadProgressState>>>,
	profile: Arc<Mutex<Option<VersionProfile>>>,
}

#[async_trait::async_trait]
impl SubTask for CoreLibTask {
	async fn execute(&self, ctx: &SubTaskContext) -> Result<(), TaskError> {
		let mut cancel = ctx.cancelled.clone();
		let profile = read_profile(&self.profile)
			.ok_or_else(|| TaskError::Failed("profile missing for core libs".into()))?;

		let mut requests = Vec::new();
		let features = Features::default();
		let os_key = current_os_key();
		let arch = current_arch();

		for lib in &profile.libraries {
			if !rule_allows(lib.rules.as_ref(), os_key, arch, &features) {
				continue;
			}
			if let Some(req) = library_request(&self.game_dir, lib, os_key, arch) {
				if !req.dest.exists() {
					requests.push(req);
				}
			}
		}

		for req in requests {
			check_cancel(&mut cancel).await?;
			tracing::info!("download core {}", req.dest.display());
			self.client
				.download(
					req,
					|p| {
						set_progress(
							&self.progress,
							format!("下载核心库 {}", self.version_id),
							p.downloaded,
							p.total,
							p.speed_bps,
							false,
						)
					},
					Some(cancel.clone()),
				)
				.await
				.map_err(|e| TaskError::Failed(e.to_string()))?;
		}

		Ok(())
	}
}

struct AssetsTask {
	client: Arc<DownloadClient>,
	game_dir: PathBuf,
	version_id: String,
	progress: Option<Arc<Mutex<DownloadProgressState>>>,
	profile: Arc<Mutex<Option<VersionProfile>>>,
}

#[async_trait::async_trait]
impl SubTask for AssetsTask {
	async fn execute(&self, ctx: &SubTaskContext) -> Result<(), TaskError> {
		let mut cancel = ctx.cancelled.clone();
		let profile = read_profile(&self.profile)
			.ok_or_else(|| TaskError::Failed("profile missing for assets".into()))?;

		let assets_id = profile.assets.as_deref().unwrap_or(&self.version_id);
		let assets_dir = self.game_dir.join("assets");
		let index_path = assets_dir.join("indexes").join(format!("{assets_id}.json"));

		if let Some(index_info) = profile.asset_index.as_ref() {
			if !index_path.exists() {
				if let Some(url) = index_info.url.as_ref() {
					check_cancel(&mut cancel).await?;
					self.client
						.download(
							DownloadRequest::new(url.clone(), index_path.clone()),
							|_| {},
							Some(cancel.clone()),
						)
						.await
						.map_err(|e| TaskError::Failed(e.to_string()))?;
				} else {
					tracing::warn!("asset index url missing, skip assets");
					return Ok(());
				}
			}
		}

		if !index_path.exists() {
			tracing::warn!(
				"asset index {} not found, skip assets",
				index_path.display()
			);
			return Ok(());
		}

		let index: AssetIndex = {
			let content = fs::read_to_string(&index_path)
				.await
				.with_context(|| format!("read asset index {}", index_path.display()))
				.map_err(|e| TaskError::Failed(e.to_string()))?;
			serde_json::from_str(&content)
				.with_context(|| "parse asset index failed")
				.map_err(|e| TaskError::Failed(e.to_string()))?
		};

		let mut requests = Vec::new();
		for asset in index.objects.values() {
			let hash = &asset.hash;
			if hash.len() < 2 {
				continue;
			}
			let subdir = &hash[..2];
			let dest = assets_dir.join("objects").join(subdir).join(hash);
			if dest.exists() {
				continue;
			}
			let url = format!(
				"https://resources.download.minecraft.net/{}/{}",
				subdir, hash
			);
			requests.push(DownloadRequest::new(url, dest));
		}

		for req in requests {
			check_cancel(&mut cancel).await?;
			tracing::debug!("download asset {}", req.dest.display());
			self.client
				.download(
					req,
					|p| {
						set_progress(
							&self.progress,
							format!("下载资源 {}", assets_id),
							p.downloaded,
							p.total,
							p.speed_bps,
							false,
						)
					},
					Some(cancel.clone()),
				)
				.await
				.map_err(|e| TaskError::Failed(e.to_string()))?;
		}

		Ok(())
	}
}

async fn resolve_version_url(version_id: &str) -> TaskResult<String> {
	const MANIFEST: &str = "https://piston-meta.mojang.com/mc/game/version_manifest.json";

	let resp = reqwest::get(MANIFEST)
		.await
		.map_err(|e| TaskError::Failed(format!("Fetch manifest failed: {e}")))?;
	let text = resp
		.text()
		.await
		.map_err(|e| TaskError::Failed(format!("Read manifest failed: {e}")))?;
	let manifest: VersionManifest = serde_json::from_str(&text)
		.map_err(|e| TaskError::Failed(format!("Parse manifest failed: {e}")))?;

	let version = manifest
		.versions
		.into_iter()
		.find(|v| v.id == version_id)
		.ok_or_else(|| {
			TaskError::Failed(format!("Version {} not found in manifest", version_id))
		})?;

	Ok(version.url)
}

async fn download_client_jar(
	client: &DownloadClient,
	profile: &VersionProfile,
	game_dir: &Path,
	version_id: &str,
	mut cancel: watch::Receiver<bool>,
	progress: Option<Arc<Mutex<DownloadProgressState>>>,
) -> TaskResult<()> {
	if let Some(downloads) = profile.downloads.as_ref() {
		if let Some(client_dl) = downloads.client.as_ref() {
			if let Some(url) = &client_dl.url {
				let dest = game_dir
					.join("versions")
					.join(version_id)
					.join(format!("{version_id}.jar"));
				if !dest.exists() {
					check_cancel(&mut cancel).await?;
					tracing::info!("download client {}", dest.display());
					client
						.download(
							DownloadRequest::new(url.clone(), dest),
							|p| {
								set_progress(
									&progress,
									format!("下载客户端 {}", version_id),
									p.downloaded,
									p.total,
									p.speed_bps,
									false,
								)
							},
							Some(cancel.clone()),
						)
						.await
						.map_err(|e| TaskError::Failed(e.to_string()))?;
				}
			}
		}
	}

	Ok(())
}

async fn download_core_libraries(
	client: &DownloadClient,
	profile: &VersionProfile,
	game_dir: &Path,
	version_id: &str,
	mut cancel: watch::Receiver<bool>,
	progress: Option<Arc<Mutex<DownloadProgressState>>>,
) -> TaskResult<()> {
	let mut requests = Vec::new();
	let features = Features::default();
	let os_key = current_os_key();
	let arch = current_arch();

	for lib in &profile.libraries {
		if !rule_allows(lib.rules.as_ref(), os_key, arch, &features) {
			continue;
		}
		if let Some(req) = library_request(game_dir, lib, os_key, arch) {
			if !req.dest.exists() {
				requests.push(req);
			}
		}
	}

	for req in requests {
		check_cancel(&mut cancel).await?;
		tracing::info!("download core {}", req.dest.display());
		client
			.download(
				req,
				|p| {
					set_progress(
						&progress,
						format!("下载核心库 {}", version_id),
						p.downloaded,
						p.total,
						p.speed_bps,
						false,
					)
				},
				Some(cancel.clone()),
			)
			.await
			.map_err(|e| TaskError::Failed(e.to_string()))?;
	}

	Ok(())
}

fn library_request(
	game_dir: &Path,
	lib: &Library,
	os_key: &str,
	_arch: &str,
) -> Option<DownloadRequest> {
	let downloads = lib.downloads.as_ref()?;

	if let Some(natives) = &lib.natives {
		if let Some(_native_cls) = natives.get(os_key) {
			if let Some(classifiers) = &downloads.classifiers {
				let key = format!("natives-{os_key}");
				if let Some(artifact) = classifiers.get(&key) {
					if let (Some(path), Some(url)) = (&artifact.path, &artifact.url) {
						let dest = game_dir
							.join("libraries")
							.join(path.replace('/', std::path::MAIN_SEPARATOR_STR));
						return Some(DownloadRequest::new(url.clone(), dest));
					}
				}
			}

			if let Some(artifact) = &downloads.artifact {
				if let Some(path) = &artifact.path {
					let dest = game_dir
						.join("libraries")
						.join(path.replace('/', std::path::MAIN_SEPARATOR_STR));
					if let Some(url) = &artifact.url {
						return Some(DownloadRequest::new(url.clone(), dest));
					}
				}
			}
		}
	}

	if let Some(artifact) = &downloads.artifact {
		if let (Some(path), Some(url)) = (&artifact.path, &artifact.url) {
			let dest = game_dir
				.join("libraries")
				.join(path.replace('/', std::path::MAIN_SEPARATOR_STR));
			return Some(DownloadRequest::new(url.clone(), dest));
		}
	}

	None
}

async fn download_assets(
	client: &DownloadClient,
	profile: &VersionProfile,
	game_dir: &Path,
	version_id: &str,
	mut cancel: watch::Receiver<bool>,
	progress: Option<Arc<Mutex<DownloadProgressState>>>,
) -> TaskResult<()> {
	let assets_id = profile.assets.as_deref().unwrap_or(version_id);
	let assets_dir = game_dir.join("assets");
	let index_path = assets_dir.join("indexes").join(format!("{assets_id}.json"));

	if let Some(index_info) = profile.asset_index.as_ref() {
		if !index_path.exists() {
			if let Some(url) = index_info.url.as_ref() {
				check_cancel(&mut cancel).await?;
				client
					.download(
						DownloadRequest::new(url.clone(), index_path.clone()),
						|_| {},
						Some(cancel.clone()),
					)
					.await
					.map_err(|e| TaskError::Failed(e.to_string()))?;
			} else {
				tracing::warn!("asset index url missing, skip assets");
				return Ok(());
			}
		}
	}

	if !index_path.exists() {
		tracing::warn!(
			"asset index {} not found, skip assets",
			index_path.display()
		);
		return Ok(());
	}

	let index: AssetIndex = {
		let content = fs::read_to_string(&index_path)
			.await
			.with_context(|| format!("read asset index {}", index_path.display()))
			.map_err(|e| TaskError::Failed(e.to_string()))?;
		serde_json::from_str(&content)
			.with_context(|| "parse asset index failed")
			.map_err(|e| TaskError::Failed(e.to_string()))?
	};

	let mut requests = Vec::new();
	for asset in index.objects.values() {
		let hash = &asset.hash;
		if hash.len() < 2 {
			continue;
		}
		let subdir = &hash[..2];
		let dest = assets_dir.join("objects").join(subdir).join(hash);
		if dest.exists() {
			continue;
		}
		let url = format!(
			"https://resources.download.minecraft.net/{}/{}",
			subdir, hash
		);
		requests.push(DownloadRequest::new(url, dest));
	}

	for req in requests {
		check_cancel(&mut cancel).await?;
		tracing::debug!("download asset {}", req.dest.display());
		client
			.download(
				req,
				|p| {
					set_progress(
						&progress,
						format!("下载资源 {}", assets_id),
						p.downloaded,
						p.total,
						p.speed_bps,
						false,
					)
				},
				Some(cancel.clone()),
			)
			.await
			.map_err(|e| TaskError::Failed(e.to_string()))?;
	}

	Ok(())
}

async fn check_cancel(cancel: &mut watch::Receiver<bool>) -> TaskResult<()> {
	if cancel.has_changed().unwrap_or(false) && *cancel.borrow() {
		return Err(TaskError::Cancelled);
	}
	if *cancel.borrow() {
		return Err(TaskError::Cancelled);
	}
	Ok(())
}

#[derive(Debug, Deserialize)]
struct AssetIndex {
	objects: HashMap<String, AssetObject>,
}

#[derive(Debug, Deserialize)]
struct AssetObject {
	hash: String,
	size: u64,
}

#[derive(Debug, Deserialize)]
struct VersionManifest {
	versions: Vec<VersionRef>,
}

#[derive(Debug, Deserialize)]
struct VersionRef {
	id: String,
	url: String,
}

#[derive(Clone, Debug, Default)]
pub struct DownloadProgressState {
	pub message: String,
	pub downloaded: u64,
	pub total: Option<u64>,
	pub speed_bps: f64,
	pub finished: bool,
}

fn set_progress(
	progress: &Option<Arc<Mutex<DownloadProgressState>>>,
	message: String,
	downloaded: u64,
	total: Option<u64>,
	speed_bps: f64,
	finished: bool,
) {
	if let Some(p) = progress {
		if let Ok(mut guard) = p.lock() {
			guard.message = message;
			guard.downloaded = downloaded;
			guard.total = total;
			guard.speed_bps = speed_bps;
			guard.finished = finished;
		}
	}
}
