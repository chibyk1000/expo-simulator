use crate::profile::{DeviceProfile, Orientation};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct DeviceManager {
    profiles: HashMap<String, DeviceProfile>,
    active_profile_id: String,
    orientation: Orientation,
}

impl Default for DeviceManager {
    fn default() -> Self {
        let mut mgr = Self {
            profiles: HashMap::new(),
            active_profile_id: "iphone-16-pro".to_string(),
            orientation: Orientation::Portrait,
        };
        let p9 = DeviceProfile::pixel_9();
        mgr.profiles.insert(p9.id.clone(), p9);
        mgr
    }
}

impl DeviceManager {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a profile from its JSON definition. Returns false if the JSON is invalid.
    pub fn load_from_json(&mut self, json: &str) -> bool {
        match serde_json::from_str::<DeviceProfile>(json) {
            Ok(profile) => {
                self.profiles.insert(profile.id.clone(), profile);
                true
            }
            Err(e) => {
                log::warn!("Invalid device profile: {}", e);
                false
            }
        }
    }

    pub fn load_from_dir<P: AsRef<Path>>(&mut self, dir: P) -> anyhow::Result<()> {
        let dir = dir.as_ref();
        if !dir.exists() {
            return Ok(());
        }

        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                let content = std::fs::read_to_string(&path)?;
                if let Ok(profile) = serde_json::from_str::<DeviceProfile>(&content) {
                    log::info!("Loaded device profile: {} ({})", profile.name, profile.id);
                    self.profiles.insert(profile.id.clone(), profile);
                }
            }
        }
        Ok(())
    }

    pub fn active_profile(&self) -> &DeviceProfile {
        self.profiles.get(&self.active_profile_id).unwrap_or_else(|| {
            self.profiles.values().next().expect("No device profiles available")
        })
    }

    pub fn set_active_profile(&mut self, id: &str) -> bool {
        if self.profiles.contains_key(id) {
            self.active_profile_id = id.to_string();
            true
        } else {
            false
        }
    }

    pub fn orientation(&self) -> Orientation {
        self.orientation
    }

    pub fn toggle_orientation(&mut self) -> Orientation {
        self.orientation = match self.orientation {
            Orientation::Portrait => Orientation::Landscape,
            Orientation::Landscape => Orientation::Portrait,
        };
        self.orientation
    }

    pub fn list_profiles(&self) -> Vec<&DeviceProfile> {
        self.profiles.values().collect()
    }
}
