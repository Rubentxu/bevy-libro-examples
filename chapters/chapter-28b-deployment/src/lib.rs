// Capítulo 28B. Deployment — Release checklist, build profiles, CI config
use std::collections::HashMap;

/// Release platform targets
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Platform {
    Windows,
    Linux,
    Macos,
    Web,
    Android,
    Ios,
}

impl Platform {
    pub fn target_triple(&self) -> &'static str {
        match self {
            Platform::Windows => "x86_64-pc-windows-msvc",
            Platform::Linux => "x86_64-unknown-linux-gnu",
            Platform::Macos => "x86_64-apple-darwin",
            Platform::Web => "wasm32-unknown-unknown",
            Platform::Android => "aarch64-linux-android",
            Platform::Ios => "aarch64-apple-ios",
        }
    }

    pub fn file_extension(&self) -> &'static str {
        match self {
            Platform::Windows => ".exe",
            Platform::Linux | Platform::Macos => "",
            Platform::Web => ".wasm",
            Platform::Android => ".apk",
            Platform::Ios => ".ipa",
        }
    }
}

/// Cargo build profile settings
#[derive(Clone, Debug)]
pub struct BuildProfile {
    pub name: String,
    pub opt_level: OptLevel,
    pub lto: LtoSetting,
    pub codegen_units: u32,
    pub strip: bool,
    pub panic: PanicStrategy,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum OptLevel {
    Off,
    Size,    // -Oz
    Speed,   // -O3
    Balanced, // -Os
}

impl OptLevel {
    pub fn cargo_value(&self) -> &'static str {
        match self {
            OptLevel::Off => "0",
            OptLevel::Size => "z",
            OptLevel::Speed => "3",
            OptLevel::Balanced => "s",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LtoSetting {
    Off,
    Thin,
    Fat,
}

impl LtoSetting {
    pub fn cargo_value(&self) -> &'static str {
        match self {
            LtoSetting::Off => "false",
            LtoSetting::Thin => "thin",
            LtoSetting::Fat => "true",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PanicStrategy {
    Unwind,
    Abort,
}

/// Pre-release checklist
pub struct ReleaseChecklist {
    pub items: Vec<ChecklistItem>,
}

#[derive(Clone, Debug)]
pub struct ChecklistItem {
    pub category: String,
    pub description: String,
    pub completed: bool,
    pub critical: bool,
}

impl ReleaseChecklist {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    pub fn add(&mut self, category: &str, description: &str, critical: bool) {
        self.items.push(ChecklistItem {
            category: category.to_string(),
            description: description.to_string(),
            completed: false,
            critical,
        });
    }

    pub fn complete(&mut self, description: &str) {
        if let Some(item) = self.items.iter_mut().find(|i| i.description == description) {
            item.completed = true;
        }
    }

    pub fn is_ready_for_release(&self) -> bool {
        self.items
            .iter()
            .filter(|i| i.critical)
            .all(|i| i.completed)
    }

    pub fn pending_critical(&self) -> Vec<&ChecklistItem> {
        self.items
            .iter()
            .filter(|i| i.critical && !i.completed)
            .collect()
    }

    pub fn progress_pct(&self) -> f32 {
        if self.items.is_empty() {
            return 100.0;
        }
        let completed = self.items.iter().filter(|i| i.completed).count();
        (completed as f32 / self.items.len() as f32) * 100.0
    }

    /// Generate a default release checklist for a Bevy game
    pub fn default_bevy() -> Self {
        let mut checklist = Self::new();

        // Code quality
        checklist.add("Code", "All tests passing (cargo test)", true);
        checklist.add("Code", "No compiler warnings (cargo build 2>&1)", false);
        checklist.add("Code", "Clippy clean (cargo clippy)", false);
        checklist.add("Code", "Code formatted (cargo fmt --check)", false);

        // Assets
        checklist.add("Assets", "All assets load correctly", true);
        checklist.add("Assets", "Audio files present and playable", true);
        checklist.add("Assets", "No placeholder/debug assets remain", false);

        // Platforms
        checklist.add("Platforms", "Builds for Windows", true);
        checklist.add("Platforms", "Builds for Linux", true);
        checklist.add("Platforms", "Builds for Web (wasm)", false);

        // Performance
        checklist.add("Performance", "Frame rate stable at 60 FPS", true);
        checklist.add("Performance", "No memory leaks in long sessions", true);
        checklist.add("Performance", "Binary size acceptable", false);

        // Polish
        checklist.add("Polish", "Game saves and loads correctly", true);
        checklist.add("Polish", "No crash on common edge cases", true);
        checklist.add("Polish", "Credits/about screen present", false);

        // Distribution
        checklist.add("Distribution", "Version number bumped", true);
        checklist.add("Distribution", "Changelog updated", false);
        checklist.add("Distribution", "README updated", false);

        checklist
    }
}

impl Default for ReleaseChecklist {
    fn default() -> Self {
        Self::new()
    }
}

/// Generate Cargo.toml [profile.release] section
pub fn generate_release_profile(profile: &BuildProfile) -> String {
    format!(
        r#"[profile.{}]
opt-level = "{}"
lto = "{}"
codegen-units = {}
strip = {}
panic = "{}"
"#,
        profile.name,
        profile.opt_level.cargo_value(),
        profile.lto.cargo_value(),
        profile.codegen_units,
        profile.strip,
        match profile.panic {
            PanicStrategy::Unwind => "unwind",
            PanicStrategy::Abort => "abort",
        },
    )
}

/// Recommended profile for web deployment
pub fn web_release_profile() -> BuildProfile {
    BuildProfile {
        name: "web".to_string(),
        opt_level: OptLevel::Size, // Optimize for small binary on web
        lto: LtoSetting::Fat,
        codegen_units: 1,
        strip: true,
        panic: PanicStrategy::Abort,
    }
}

/// Recommended profile for desktop release
pub fn desktop_release_profile() -> BuildProfile {
    BuildProfile {
        name: "release".to_string(),
        opt_level: OptLevel::Speed,
        lto: LtoSetting::Thin,
        codegen_units: 16,
        strip: true,
        panic: PanicStrategy::Unwind, // Need unwinding for crash reports
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn platform_target_triples() {
        assert_eq!(Platform::Windows.target_triple(), "x86_64-pc-windows-msvc");
        assert_eq!(Platform::Web.target_triple(), "wasm32-unknown-unknown");
        assert_eq!(Platform::Linux.target_triple(), "x86_64-unknown-linux-gnu");
    }

    #[test]
    fn platform_file_extensions() {
        assert_eq!(Platform::Windows.file_extension(), ".exe");
        assert_eq!(Platform::Web.file_extension(), ".wasm");
        assert_eq!(Platform::Android.file_extension(), ".apk");
    }

    #[test]
    fn opt_level_cargo_values() {
        assert_eq!(OptLevel::Size.cargo_value(), "z");
        assert_eq!(OptLevel::Speed.cargo_value(), "3");
        assert_eq!(OptLevel::Balanced.cargo_value(), "s");
        assert_eq!(OptLevel::Off.cargo_value(), "0");
    }

    #[test]
    fn lto_cargo_values() {
        assert_eq!(LtoSetting::Thin.cargo_value(), "thin");
        assert_eq!(LtoSetting::Fat.cargo_value(), "true");
        assert_eq!(LtoSetting::Off.cargo_value(), "false");
    }

    #[test]
    fn release_checklist_default() {
        let checklist = ReleaseChecklist::default_bevy();
        assert!(!checklist.items.is_empty());
        assert!(checklist.items.iter().any(|i| i.critical));
    }

    #[test]
    fn checklist_not_ready_initially() {
        let checklist = ReleaseChecklist::default_bevy();
        assert!(!checklist.is_ready_for_release());
        assert!(!checklist.pending_critical().is_empty());
    }

    #[test]
    fn checklist_complete_item() {
        let mut checklist = ReleaseChecklist::default_bevy();
        let critical_count = checklist.pending_critical().len();

        checklist.complete("All tests passing (cargo test)");

        assert!(checklist.pending_critical().len() < critical_count);
    }

    #[test]
    fn checklist_ready_when_all_critical_done() {
        let mut checklist = ReleaseChecklist::new();
        checklist.add("Test", "Critical item 1", true);
        checklist.add("Test", "Critical item 2", true);
        checklist.add("Test", "Optional item", false);

        assert!(!checklist.is_ready_for_release());

        checklist.complete("Critical item 1");
        checklist.complete("Critical item 2");

        assert!(checklist.is_ready_for_release(), "Should be ready when all critical done");
    }

    #[test]
    fn checklist_progress() {
        let mut checklist = ReleaseChecklist::new();
        checklist.add("T", "Item 1", false);
        checklist.add("T", "Item 2", false);
        checklist.add("T", "Item 3", false);
        checklist.add("T", "Item 4", false);

        assert_eq!(checklist.progress_pct(), 0.0);

        checklist.complete("Item 1");
        checklist.complete("Item 2");

        assert_eq!(checklist.progress_pct(), 50.0);
    }

    #[test]
    fn generate_release_profile_string() {
        let profile = BuildProfile {
            name: "release".to_string(),
            opt_level: OptLevel::Speed,
            lto: LtoSetting::Thin,
            codegen_units: 16,
            strip: true,
            panic: PanicStrategy::Unwind,
        };

        let toml = generate_release_profile(&profile);
        assert!(toml.contains("[profile.release]"));
        assert!(toml.contains("opt-level = \"3\""));
        assert!(toml.contains("lto = \"thin\""));
        assert!(toml.contains("codegen-units = 16"));
        assert!(toml.contains("strip = true"));
        assert!(toml.contains("panic = \"unwind\""));
    }

    #[test]
    fn web_profile_optimizes_for_size() {
        let profile = web_release_profile();
        assert_eq!(profile.opt_level, OptLevel::Size);
        assert_eq!(profile.lto, LtoSetting::Fat);
        assert_eq!(profile.codegen_units, 1);
        assert_eq!(profile.panic, PanicStrategy::Abort);
    }

    #[test]
    fn desktop_profile_optimizes_for_speed() {
        let profile = desktop_release_profile();
        assert_eq!(profile.opt_level, OptLevel::Speed);
        assert_eq!(profile.lto, LtoSetting::Thin);
        assert!(profile.codegen_units > 1);
        assert_eq!(profile.panic, PanicStrategy::Unwind);
    }
}
