// Capítulo 27B. Versioning — Semantic versioning, migration guides
use std::cmp::Ordering;

/// Semantic version: MAJOR.MINOR.PATCH
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SemVer {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl SemVer {
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self { major, minor, patch }
    }

    pub fn parse(s: &str) -> Result<Self, String> {
        let s = s.trim().trim_start_matches('v');
        let parts: Vec<&str> = s.split('.').collect();
        if parts.len() != 3 {
            return Err(format!("Expected MAJOR.MINOR.PATCH, got '{}'", s));
        }
        Ok(Self {
            major: parts[0].parse().map_err(|e| format!("Invalid major: {}", e))?,
            minor: parts[1].parse().map_err(|e| format!("Invalid minor: {}", e))?,
            patch: parts[2].parse().map_err(|e| format!("Invalid patch: {}", e))?,
        })
    }

    pub fn to_string(&self) -> String {
        format!("{}.{}.{}", self.major, self.minor, self.patch)
    }

    /// What kind of change does going from `from` to `self` represent?
    pub fn change_type(&self, from: &SemVer) -> ChangeType {
        match (
            self.major.cmp(&from.major),
            self.minor.cmp(&from.minor),
            self.patch.cmp(&from.patch),
        ) {
            (Ordering::Greater, _, _) => ChangeType::Major,
            (Ordering::Equal, Ordering::Greater, _) => ChangeType::Minor,
            (Ordering::Equal, Ordering::Equal, Ordering::Greater) => ChangeType::Patch,
            (Ordering::Equal, Ordering::Equal, Ordering::Equal) => ChangeType::None,
            _ => ChangeType::Downgrade,
        }
    }

    pub fn is_compatible_with(&self, other: &SemVer) -> bool {
        // Compatible if same major version (semver rule)
        self.major == other.major
    }

    /// Check if this version is at least `other` (i.e., self >= other)
    pub fn at_least(&self, other: &SemVer) -> bool {
        match self.major.cmp(&other.major) {
            Ordering::Greater => true,
            Ordering::Less => false,
            Ordering::Equal => match self.minor.cmp(&other.minor) {
                Ordering::Greater => true,
                Ordering::Less => false,
                Ordering::Equal => self.patch >= other.patch,
            },
        }
    }
}

impl Ord for SemVer {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.major.cmp(&other.major) {
            Ordering::Equal => match self.minor.cmp(&other.minor) {
                Ordering::Equal => self.patch.cmp(&other.patch),
                other => other,
            },
            other => other,
        }
    }
}

impl PartialOrd for SemVer {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChangeType {
    Major,    // Breaking change
    Minor,    // New feature, backwards compatible
    Patch,    // Bug fix, backwards compatible
    None,     // No change
    Downgrade, // Version went backwards
}

impl ChangeType {
    pub fn requires_migration(&self) -> bool {
        matches!(self, ChangeType::Major)
    }

    pub fn description(&self) -> &'static str {
        match self {
            ChangeType::Major => "Breaking change — migration required",
            ChangeType::Minor => "New feature — backwards compatible",
            ChangeType::Patch => "Bug fix — backwards compatible",
            ChangeType::None => "No change",
            ChangeType::Downgrade => "Version downgraded (unusual)",
        }
    }
}

/// Version requirement range (e.g., ">=0.18.0, <0.20.0")
#[derive(Clone, Debug)]
pub struct VersionRange {
    pub min: Option<SemVer>,
    pub max: Option<SemVer>,
}

impl VersionRange {
    pub fn satisfies(&self, version: &SemVer) -> bool {
        if let Some(ref min) = self.min {
            if !version.at_least(min) {
                return false;
            }
        }
        if let Some(ref max) = self.max {
            if version.at_least(max) && version != max {
                return false;
            }
        }
        true
    }
}

/// Migration step for breaking changes
#[derive(Clone, Debug)]
pub struct MigrationStep {
    pub from_version: SemVer,
    pub to_version: SemVer,
    pub description: String,
    pub code_before: String,
    pub code_after: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn semver_parse() {
        let v = SemVer::parse("1.2.3").unwrap();
        assert_eq!(v, SemVer::new(1, 2, 3));
    }

    #[test]
    fn semver_parse_with_v() {
        let v = SemVer::parse("v0.19.0").unwrap();
        assert_eq!(v, SemVer::new(0, 19, 0));
    }

    #[test]
    fn semver_parse_invalid() {
        assert!(SemVer::parse("1.2").is_err());
        assert!(SemVer::parse("abc").is_err());
    }

    #[test]
    fn semver_ordering() {
        let v1 = SemVer::new(1, 0, 0);
        let v2 = SemVer::new(1, 0, 1);
        let v3 = SemVer::new(1, 1, 0);
        let v4 = SemVer::new(2, 0, 0);

        assert!(v1 < v2);
        assert!(v2 < v3);
        assert!(v3 < v4);
    }

    #[test]
    fn change_type_major() {
        let from = SemVer::new(1, 0, 0);
        let to = SemVer::new(2, 0, 0);
        assert_eq!(to.change_type(&from), ChangeType::Major);
        assert!(to.change_type(&from).requires_migration());
    }

    #[test]
    fn change_type_minor() {
        let from = SemVer::new(0, 18, 0);
        let to = SemVer::new(0, 19, 0);
        assert_eq!(to.change_type(&from), ChangeType::Minor);
        assert!(!to.change_type(&from).requires_migration());
    }

    #[test]
    fn change_type_patch() {
        let from = SemVer::new(0, 19, 0);
        let to = SemVer::new(0, 19, 1);
        assert_eq!(to.change_type(&from), ChangeType::Patch);
    }

    #[test]
    fn change_type_none() {
        let v = SemVer::new(1, 0, 0);
        assert_eq!(v.change_type(&v), ChangeType::None);
    }

    #[test]
    fn semver_compatible() {
        let v1 = SemVer::new(0, 19, 0);
        let v2 = SemVer::new(0, 19, 5);
        let v3 = SemVer::new(0, 20, 0);
        let v4 = SemVer::new(1, 0, 0);

        // Same major = compatible (even if different minor)
        assert!(v1.is_compatible_with(&v2));
        assert!(v1.is_compatible_with(&v3));

        // Different major = incompatible
        assert!(!v1.is_compatible_with(&v4));
    }

    #[test]
    fn version_range_satisfies() {
        let range = VersionRange {
            min: Some(SemVer::new(0, 18, 0)),
            max: Some(SemVer::new(0, 19, u32::MAX)),
        };

        // 0.18.0 should satisfy
        assert!(range.satisfies(&SemVer::new(0, 18, 0)));
        // 0.19.0 should satisfy
        assert!(range.satisfies(&SemVer::new(0, 19, 0)));
        // 0.17.0 should NOT satisfy (too old)
        assert!(!range.satisfies(&SemVer::new(0, 17, 0)));
    }

    #[test]
    fn at_least() {
        let v = SemVer::new(0, 19, 0);
        assert!(v.at_least(&SemVer::new(0, 18, 0)));
        assert!(v.at_least(&SemVer::new(0, 19, 0)));
        assert!(!v.at_least(&SemVer::new(0, 20, 0)));
        assert!(!v.at_least(&SemVer::new(1, 0, 0)));
    }

    #[test]
    fn migration_step_example() {
        let step = MigrationStep {
            from_version: SemVer::new(0, 18, 0),
            to_version: SemVer::new(0, 19, 0),
            description: "Trigger<E> renamed to On<E>".to_string(),
            code_before: "fn observer(trigger: Trigger<MyEvent>)".to_string(),
            code_after: "fn observer(on: On<MyEvent>)".to_string(),
        };

        assert_eq!(step.to_version.change_type(&step.from_version), ChangeType::Minor);
        assert!(step.description.contains("Trigger"));
        assert!(step.code_after.contains("On<"));
    }

    #[test]
    fn bevy_version_compatibility() {
        // Bevy 0.18 → 0.19 is a minor bump (0.x ecosystem)
        let bevy_18 = SemVer::new(0, 18, 0);
        let bevy_19 = SemVer::new(0, 19, 0);

        assert!(bevy_19.change_type(&bevy_18) == ChangeType::Minor);
        assert!(bevy_19.is_compatible_with(&bevy_18)); // Same major (0)

        // In practice, Bevy 0.x minor bumps are often breaking
        // (pre-1.0 semver convention), but semver says they're compatible
    }
}
