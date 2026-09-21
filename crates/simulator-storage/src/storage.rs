use std::path::{Path, PathBuf};
use std::fs;

#[derive(Debug, Clone)]
pub struct DeviceStorage {
    device_id: String,
    base_dir: PathBuf,
}

impl DeviceStorage {
    pub fn new(device_id: &str) -> Self {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let base_dir = PathBuf::from(home)
            .join(".expo-sim")
            .join("devices")
            .join(device_id);

        let storage = Self {
            device_id: device_id.to_string(),
            base_dir,
        };
        let _ = storage.ensure_dirs();
        storage
    }

    pub fn with_base_dir<P: AsRef<Path>>(device_id: &str, base: P) -> Self {
        let storage = Self {
            device_id: device_id.to_string(),
            base_dir: base.as_ref().join("devices").join(device_id),
        };
        let _ = storage.ensure_dirs();
        storage
    }

    pub fn ensure_dirs(&self) -> std::io::Result<()> {
        fs::create_dir_all(self.filesystem_dir())?;
        fs::create_dir_all(self.preferences_dir())?;
        fs::create_dir_all(self.databases_dir())?;
        fs::create_dir_all(self.cache_dir())?;
        fs::create_dir_all(self.secure_store_dir())?;
        Ok(())
    }

    pub fn device_id(&self) -> &str {
        &self.device_id
    }

    pub fn filesystem_dir(&self) -> PathBuf {
        self.base_dir.join("filesystem")
    }

    pub fn preferences_dir(&self) -> PathBuf {
        self.base_dir.join("preferences")
    }

    pub fn databases_dir(&self) -> PathBuf {
        self.base_dir.join("databases")
    }

    pub fn cache_dir(&self) -> PathBuf {
        self.base_dir.join("cache")
    }

    pub fn secure_store_dir(&self) -> PathBuf {
        self.base_dir.join("secure-store")
    }
}
