use crate::error::{Error, Result};
use std::collections::HashMap;

pub trait AssetProvider {
	fn load_bytes(&self, path: &str) -> Result<Vec<u8>>;
}

pub struct FileAssetProvider {
	base_path: String,
}

impl FileAssetProvider {
	pub fn new(base_path: &str) -> Self {
		Self {
			base_path: base_path.trim_end_matches('/').to_string(),
		}
	}

	fn join_path(&self, path: &str) -> String {
		if path.starts_with(&self.base_path) {
			return path.to_string();
		}
		format!("{}/{}", self.base_path, path)
	}
}

impl AssetProvider for FileAssetProvider {
	fn load_bytes(&self, path: &str) -> Result<Vec<u8>> {
		let full_path = self.join_path(path);
		std::fs::read(&full_path).map_err(Error::Io)
	}
}

pub struct MemoryAssetProvider {
	assets: HashMap<String, Vec<u8>>,
}

impl MemoryAssetProvider {
	pub fn new(asset_paths: Vec<String>, asset_bytes: Vec<Vec<u8>>) -> Result<Self> {
		if asset_paths.len() != asset_bytes.len() {
			return Err(Error::InvalidData(
				"asset_paths and asset_bytes length mismatch".to_string(),
			));
		}

		let mut assets = HashMap::new();
		for (path, bytes) in asset_paths.into_iter().zip(asset_bytes.into_iter()) {
			assets.insert(path, bytes);
		}

		Ok(Self { assets })
	}
}

impl AssetProvider for MemoryAssetProvider {
	fn load_bytes(&self, path: &str) -> Result<Vec<u8>> {
		self.assets
			.get(path)
			.cloned()
			.ok_or_else(|| Error::InvalidData(format!("Asset not provided: {}", path)))
	}
}
