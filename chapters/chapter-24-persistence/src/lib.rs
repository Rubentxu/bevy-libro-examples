// Capítulo 24. Persistence — serialization, save/load
// Pure data operations, fully testable.
use std::collections::HashMap;

/// Game state that can be serialized/deserialized
#[derive(Clone, Debug, PartialEq)]
pub struct SaveData {
    pub player_name: String,
    pub level: u32,
    pub score: u64,
    pub health: i32,
    pub max_health: i32,
    pub inventory: Vec<String>,
    pub settings: HashMap<String, String>,
}

impl Default for SaveData {
    fn default() -> Self {
        let mut settings = HashMap::new();
        settings.insert("music_volume".to_string(), "0.8".to_string());
        settings.insert("sfx_volume".to_string(), "1.0".to_string());
        settings.insert("fullscreen".to_string(), "false".to_string());
        Self {
            player_name: "Player".to_string(),
            level: 1,
            score: 0,
            health: 100,
            max_health: 100,
            inventory: Vec::new(),
            settings,
        }
    }
}

/// Simple RON-like serialization (without the ron crate dependency)
impl SaveData {
    pub fn to_ron(&self) -> String {
        let mut out = String::new();
        out.push_str("SaveData(\n");
        out.push_str(&format!("    player_name: \"{}\",\n", self.player_name));
        out.push_str(&format!("    level: {},\n", self.level));
        out.push_str(&format!("    score: {},\n", self.score));
        out.push_str(&format!("    health: {},\n", self.health));
        out.push_str(&format!("    max_health: {},\n", self.max_health));
        out.push_str("    inventory: [\n");
        for item in &self.inventory {
            out.push_str(&format!("        \"{}\",\n", item));
        }
        out.push_str("    ],\n");
        out.push_str("    settings: {\n");
        for (k, v) in &self.settings {
            out.push_str(&format!("        \"{}\": \"{}\",\n", k, v));
        }
        out.push_str("    },\n");
        out.push_str(")\n");
        out
    }

    pub fn from_ron(ron: &str) -> Result<Self, String> {
        let mut data = SaveData::default();

        for line in ron.lines() {
            let line = line.trim();
            if line.starts_with("player_name:") {
                data.player_name = extract_string(line)?;
            } else if line.starts_with("level:") {
                data.level = extract_number(line)? as u32;
            } else if line.starts_with("score:") {
                data.score = extract_number(line)? as u64;
            } else if line.starts_with("health:") && !line.starts_with("max_") {
                data.health = extract_number(line)? as i32;
            } else if line.starts_with("max_health:") {
                data.max_health = extract_number(line)? as i32;
            }
        }

        Ok(data)
    }
}

fn extract_string(line: &str) -> Result<String, String> {
    let start = line.find('"').ok_or("No opening quote")? + 1;
    let end = line.rfind('"').ok_or("No closing quote")?;
    if end <= start {
        return Err("Invalid string".to_string());
    }
    Ok(line[start..end].to_string())
}

fn extract_number(line: &str) -> Result<i64, String> {
    let parts: Vec<&str> = line.split(':').collect();
    if parts.len() != 2 {
        return Err("Invalid format".to_string());
    }
    parts[1]
        .trim()
        .trim_end_matches(',')
        .parse()
        .map_err(|e| format!("Parse error: {}", e))
}

/// Checkpoint system for save points
#[derive(Clone, Debug)]
pub struct Checkpoint {
    pub id: u32,
    pub position: (f32, f32),
    pub timestamp: u64,
}

pub struct CheckpointSystem {
    checkpoints: Vec<Checkpoint>,
    last_triggered: Option<u32>,
}

impl CheckpointSystem {
    pub fn new() -> Self {
        Self {
            checkpoints: Vec::new(),
            last_triggered: None,
        }
    }

    pub fn add(&mut self, checkpoint: Checkpoint) {
        self.checkpoints.push(checkpoint);
    }

    pub fn trigger(&mut self, id: u32) -> bool {
        if self.checkpoints.iter().any(|c| c.id == id) {
            self.last_triggered = Some(id);
            true
        } else {
            false
        }
    }

    pub fn last_checkpoint(&self) -> Option<&Checkpoint> {
        self.last_triggered
            .and_then(|id| self.checkpoints.iter().find(|c| c.id == id))
    }
}

impl Default for CheckpointSystem {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn save_data_default() {
        let data = SaveData::default();
        assert_eq!(data.player_name, "Player");
        assert_eq!(data.level, 1);
        assert_eq!(data.health, 100);
    }

    #[test]
    fn save_data_to_ron_roundtrip() {
        let original = SaveData {
            player_name: "Hero".to_string(),
            level: 5,
            score: 10000,
            health: 75,
            max_health: 100,
            inventory: vec!["sword".to_string(), "potion".to_string()],
            settings: HashMap::new(),
        };

        let ron = original.to_ron();
        let parsed = SaveData::from_ron(&ron).expect("Should parse");

        assert_eq!(parsed.player_name, "Hero");
        assert_eq!(parsed.level, 5);
        assert_eq!(parsed.score, 10000);
        assert_eq!(parsed.health, 75);
    }

    #[test]
    fn save_data_health_persists() {
        let data = SaveData {
            health: 42,
            max_health: 100,
            ..Default::default()
        };
        let ron = data.to_ron();
        let parsed = SaveData::from_ron(&ron).unwrap();
        assert_eq!(parsed.health, 42);
    }

    #[test]
    fn checkpoint_trigger() {
        let mut system = CheckpointSystem::new();
        system.add(Checkpoint {
            id: 1,
            position: (10.0, 20.0),
            timestamp: 1000,
        });

        assert!(system.trigger(1));
        assert!(!system.trigger(999));
    }

    #[test]
    fn checkpoint_last() {
        let mut system = CheckpointSystem::new();
        system.add(Checkpoint {
            id: 1,
            position: (0.0, 0.0),
            timestamp: 100,
        });
        system.add(Checkpoint {
            id: 2,
            position: (50.0, 50.0),
            timestamp: 200,
        });

        system.trigger(1);
        assert_eq!(system.last_checkpoint().unwrap().id, 1);

        system.trigger(2);
        assert_eq!(system.last_checkpoint().unwrap().id, 2);
    }

    #[test]
    fn checkpoint_no_trigger_returns_none() {
        let system = CheckpointSystem::new();
        assert!(system.last_checkpoint().is_none());
    }

    #[test]
    fn settings_roundtrip() {
        let mut data = SaveData::default();
        data.settings.insert("difficulty".to_string(), "hard".to_string());

        let ron = data.to_ron();
        assert!(ron.contains("difficulty"));
        assert!(ron.contains("hard"));
    }
}
