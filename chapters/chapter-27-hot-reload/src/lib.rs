// Capítulo 27. Hot Reload — Asset versioning, change detection
use std::collections::HashMap;
use std::path::Path;

/// Asset version tracker: detects when files change
pub struct AssetWatcher {
    pub paths: HashMap<String, u64>,
}

impl AssetWatcher {
    pub fn new() -> Self {
        Self { paths: HashMap::new() }
    }

    pub fn register(&mut self, path: &str, version: u64) {
        self.paths.insert(path.to_string(), version);
    }

    pub fn check_update(&mut self, path: &str, new_version: u64) -> bool {
        match self.paths.get(path) {
            Some(&old) if old != new_version => {
                self.paths.insert(path.to_string(), new_version);
                true
            }
            None => {
                self.paths.insert(path.to_string(), new_version);
                false // First registration, not an update
            }
            _ => false,
        }
    }

    pub fn version(&self, path: &str) -> Option<u64> {
        self.paths.get(path).copied()
    }
}

impl Default for AssetWatcher {
    fn default() -> Self { Self::new() }
}

/// Scene reload coordinator
pub struct ReloadCoordinator {
    watcher: AssetWatcher,
    reload_count: u32,
}

impl ReloadCoordinator {
    pub fn new() -> Self {
        Self {
            watcher: AssetWatcher::new(),
            reload_count: 0,
        }
    }

    pub fn register_asset(&mut self, path: &str, version: u64) {
        self.watcher.register(path, version);
    }

    /// Check if any registered asset changed. Returns list of changed paths.
    pub fn check_all(&mut self, current_versions: &HashMap<String, u64>) -> Vec<String> {
        let mut changed = Vec::new();
        for (path, &new_version) in current_versions {
            if self.watcher.check_update(path, new_version) {
                changed.push(path.clone());
            }
        }
        if !changed.is_empty() {
            self.reload_count += 1;
        }
        changed
    }

    pub fn reload_count(&self) -> u32 {
        self.reload_count
    }
}

impl Default for ReloadCoordinator {
    fn default() -> Self { Self::new() }
}

/// Mod loading: validate mod structure
pub struct ModLoader {
    pub loaded_mods: Vec<String>,
}

impl ModLoader {
    pub fn new() -> Self {
        Self { loaded_mods: Vec::new() }
    }

    pub fn validate_mod(&self, mod_path: &str) -> Result<ModInfo, String> {
        let path = Path::new(mod_path);

        if !path.exists() {
            return Err(format!("Mod path does not exist: {}", mod_path));
        }

        let name = path.file_stem()
            .and_then(|s| s.to_str())
            .ok_or("Invalid mod filename")?
            .to_string();

        // Check for mod.toml or mod.json
        let has_manifest = path.join("mod.toml").exists()
            || path.join("mod.json").exists();

        if !has_manifest {
            return Err("Mod missing manifest file (mod.toml or mod.json)".to_string());
        }

        Ok(ModInfo {
            name,
            path: mod_path.to_string(),
            version: "0.1.0".to_string(),
        })
    }

    pub fn load_mod(&mut self, info: &ModInfo) {
        if !self.loaded_mods.contains(&info.name) {
            self.loaded_mods.push(info.name.clone());
        }
    }

    pub fn is_loaded(&self, name: &str) -> bool {
        self.loaded_mods.iter().any(|m| m == name)
    }

    pub fn unload_mod(&mut self, name: &str) {
        self.loaded_mods.retain(|m| m != name);
    }
}

impl Default for ModLoader {
    fn default() -> Self { Self::new() }
}

#[derive(Clone, Debug)]
pub struct ModInfo {
    pub name: String,
    pub path: String,
    pub version: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn asset_watcher_detects_change() {
        let mut watcher = AssetWatcher::new();
        watcher.register("sprites/player.png", 1);

        assert!(!watcher.check_update("sprites/player.png", 1), "Same version = no update");
        assert!(watcher.check_update("sprites/player.png", 2), "New version = update");
    }

    #[test]
    fn asset_watcher_new_registration() {
        let mut watcher = AssetWatcher::new();
        assert!(!watcher.check_update("new.asset", 1), "First registration = not an update");
    }

    #[test]
    fn asset_watcher_version() {
        let mut watcher = AssetWatcher::new();
        watcher.register("test.png", 42);
        assert_eq!(watcher.version("test.png"), Some(42));
        assert_eq!(watcher.version("missing.png"), None);
    }

    #[test]
    fn reload_coordinator_detects_multiple() {
        let mut coord = ReloadCoordinator::new();
        coord.register_asset("a.png", 1);
        coord.register_asset("b.png", 1);

        let mut versions = HashMap::new();
        versions.insert("a.png".to_string(), 2);
        versions.insert("b.png".to_string(), 2);

        let changed = coord.check_all(&versions);

        assert_eq!(changed.len(), 2, "Both assets changed");
        assert_eq!(coord.reload_count(), 1);
    }

    #[test]
    fn reload_coordinator_no_changes() {
        let mut coord = ReloadCoordinator::new();
        coord.register_asset("a.png", 1);

        let mut versions = HashMap::new();
        versions.insert("a.png".to_string(), 1);

        let changed = coord.check_all(&versions);
        assert!(changed.is_empty());
        assert_eq!(coord.reload_count(), 0);
    }

    #[test]
    fn reload_coordinator_increments_count() {
        let mut coord = ReloadCoordinator::new();
        coord.register_asset("a.png", 1);

        let mut versions = HashMap::new();
        versions.insert("a.png".to_string(), 2);
        coord.check_all(&versions);
        coord.check_all(&versions);

        // Second check shouldn't increment (version already updated)
        assert_eq!(coord.reload_count(), 1);
    }

    #[test]
    fn mod_loader_load_unload() {
        let mut loader = ModLoader::new();
        let info = ModInfo {
            name: "test_mod".to_string(),
            path: "/tmp/test".to_string(),
            version: "1.0".to_string(),
        };

        loader.load_mod(&info);
        assert!(loader.is_loaded("test_mod"));

        loader.unload_mod("test_mod");
        assert!(!loader.is_loaded("test_mod"));
    }

    #[test]
    fn mod_loader_no_duplicate_load() {
        let mut loader = ModLoader::new();
        let info = ModInfo {
            name: "test".to_string(),
            path: "/tmp".to_string(),
            version: "1.0".to_string(),
        };

        loader.load_mod(&info);
        loader.load_mod(&info);

        assert_eq!(loader.loaded_mods.len(), 1, "Should not duplicate");
    }

    #[test]
    fn mod_validate_nonexistent() {
        let loader = ModLoader::new();
        let result = loader.validate_mod("/nonexistent/path");
        assert!(result.is_err());
    }

    #[test]
    fn mod_validate_with_manifest() {
        let tmp_dir = "/tmp/opencode/test_mod_valid";
        fs::create_dir_all(tmp_dir).unwrap();
        fs::write(format!("{}/mod.toml", tmp_dir), "[mod]\nname = \"test\"").unwrap();

        let loader = ModLoader::new();
        let result = loader.validate_mod(tmp_dir);

        assert!(result.is_ok());
        let info = result.unwrap();
        assert_eq!(info.name, "test_mod_valid");

        fs::remove_dir_all(tmp_dir).ok();
    }

    #[test]
    fn mod_validate_missing_manifest() {
        let tmp_dir = "/tmp/opencode/test_mod_no_manifest";
        fs::create_dir_all(tmp_dir).unwrap();

        let loader = ModLoader::new();
        let result = loader.validate_mod(tmp_dir);
        assert!(result.is_err());

        fs::remove_dir_all(tmp_dir).ok();
    }
}
