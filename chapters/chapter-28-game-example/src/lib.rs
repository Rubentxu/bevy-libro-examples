// Capítulo 28. Game Example — Full game state machine, scene flow
use std::collections::HashMap;

/// Game scene: each represents a distinct game state
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum GameScene {
    Boot,
    TitleScreen,
    LevelSelect,
    Playing,
    Paused,
    GameOver,
    Victory,
    Credits,
}

/// Scene flow: defines valid transitions between scenes
pub struct SceneFlow {
    transitions: HashMap<GameScene, Vec<GameScene>>,
}

impl Default for SceneFlow {
    fn default() -> Self {
        let mut transitions = HashMap::new();

        transitions.insert(GameScene::Boot, vec![GameScene::TitleScreen]);
        transitions.insert(GameScene::TitleScreen, vec![GameScene::LevelSelect, GameScene::Credits]);
        transitions.insert(GameScene::LevelSelect, vec![GameScene::Playing, GameScene::TitleScreen]);
        transitions.insert(GameScene::Playing, vec![GameScene::Paused, GameScene::GameOver, GameScene::Victory]);
        transitions.insert(GameScene::Paused, vec![GameScene::Playing, GameScene::TitleScreen]);
        transitions.insert(GameScene::GameOver, vec![GameScene::TitleScreen, GameScene::LevelSelect]);
        transitions.insert(GameScene::Victory, vec![GameScene::TitleScreen, GameScene::Credits, GameScene::LevelSelect]);
        transitions.insert(GameScene::Credits, vec![GameScene::TitleScreen]);

        Self { transitions }
    }
}

impl SceneFlow {
    pub fn can_transition(&self, from: GameScene, to: GameScene) -> bool {
        self.transitions
            .get(&from)
            .map(|valid| valid.contains(&to))
            .unwrap_or(false)
    }

    pub fn valid_targets(&self, from: GameScene) -> Vec<GameScene> {
        self.transitions.get(&from).cloned().unwrap_or_default()
    }
}

/// Level data
#[derive(Clone, Debug)]
pub struct Level {
    pub id: u32,
    pub name: String,
    pub difficulty: u32,
    pub completed: bool,
    pub unlocked: bool,
}

/// Level select manager
pub struct LevelManager {
    pub levels: Vec<Level>,
}

impl LevelManager {
    pub fn new(count: u32) -> Self {
        let levels = (0..count)
            .map(|i| Level {
                id: i,
                name: format!("Level {}", i + 1),
                difficulty: (i / 3) + 1,
                completed: false,
                unlocked: i == 0, // Only first level unlocked initially
            })
            .collect();
        Self { levels }
    }

    pub fn complete_level(&mut self, id: u32) {
        if let Some(level) = self.levels.iter_mut().find(|l| l.id == id) {
            level.completed = true;

            // Unlock next level
            if let Some(next) = self.levels.iter_mut().find(|l| l.id == id + 1) {
                next.unlocked = true;
            }
        }
    }

    pub fn is_unlocked(&self, id: u32) -> bool {
        self.levels.iter().find(|l| l.id == id).map(|l| l.unlocked).unwrap_or(false)
    }

    pub fn completed_count(&self) -> usize {
        self.levels.iter().filter(|l| l.completed).count()
    }

    pub fn progress_pct(&self) -> f32 {
        if self.levels.is_empty() { return 0.0; }
        (self.completed_count() as f32 / self.levels.len() as f32) * 100.0
    }

    pub fn all_completed(&self) -> bool {
        self.levels.iter().all(|l| l.completed)
    }
}

/// Score tracker
pub struct ScoreTracker {
    pub scores: HashMap<u32, u64>, // level_id -> best_score
    pub total_score: u64,
}

impl ScoreTracker {
    pub fn new() -> Self {
        Self {
            scores: HashMap::new(),
            total_score: 0,
        }
    }

    pub fn record_score(&mut self, level_id: u32, score: u64) -> bool {
        let is_new_best = self.scores.get(&level_id).copied().unwrap_or(0) < score;
        if is_new_best {
            let old = self.scores.insert(level_id, score);
            if let Some(old_score) = old {
                self.total_score -= old_score;
            }
            self.total_score += score;
        }
        is_new_best
    }

    pub fn best_score(&self, level_id: u32) -> u64 {
        self.scores.get(&level_id).copied().unwrap_or(0)
    }
}

impl Default for ScoreTracker {
    fn default() -> Self { Self::new() }
}

/// Game state container
pub struct GameState {
    pub current_scene: GameScene,
    pub level_manager: LevelManager,
    pub score_tracker: ScoreTracker,
    pub flow: SceneFlow,
}

impl GameState {
    pub fn new() -> Self {
        Self {
            current_scene: GameScene::Boot,
            level_manager: LevelManager::new(10),
            score_tracker: ScoreTracker::new(),
            flow: SceneFlow::default(),
        }
    }

    pub fn transition_to(&mut self, target: GameScene) -> bool {
        if self.flow.can_transition(self.current_scene, target) {
            self.current_scene = target;
            true
        } else {
            false
        }
    }

    pub fn start_level(&mut self, level_id: u32) -> bool {
        if !self.level_manager.is_unlocked(level_id) {
            return false;
        }
        self.transition_to(GameScene::Playing)
    }

    pub fn complete_level(&mut self, level_id: u32, score: u64) {
        self.level_manager.complete_level(level_id);
        self.score_tracker.record_score(level_id, score);

        if self.level_manager.all_completed() {
            self.transition_to(GameScene::Victory);
        } else {
            self.transition_to(GameScene::LevelSelect);
        }
    }
}

impl Default for GameState {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scene_flow_boot_to_title() {
        let flow = SceneFlow::default();
        assert!(flow.can_transition(GameScene::Boot, GameScene::TitleScreen));
    }

    #[test]
    fn scene_flow_invalid_transition() {
        let flow = SceneFlow::default();
        assert!(!flow.can_transition(GameScene::Boot, GameScene::Playing));
    }

    #[test]
    fn scene_flow_playing_to_paused() {
        let flow = SceneFlow::default();
        assert!(flow.can_transition(GameScene::Playing, GameScene::Paused));
    }

    #[test]
    fn scene_flow_paused_to_playing() {
        let flow = SceneFlow::default();
        assert!(flow.can_transition(GameScene::Paused, GameScene::Playing));
    }

    #[test]
    fn level_manager_starts_with_one_unlocked() {
        let manager = LevelManager::new(5);
        assert!(manager.is_unlocked(0));
        assert!(!manager.is_unlocked(1));
    }

    #[test]
    fn level_manager_unlocks_next_on_complete() {
        let mut manager = LevelManager::new(5);
        manager.complete_level(0);
        assert!(manager.is_unlocked(1));
    }

    #[test]
    fn level_manager_progress() {
        let mut manager = LevelManager::new(10);
        manager.complete_level(0);
        manager.complete_level(1);
        manager.complete_level(2);

        assert_eq!(manager.completed_count(), 3);
        assert!((manager.progress_pct() - 30.0).abs() < 0.001);
    }

    #[test]
    fn level_manager_all_completed() {
        let mut manager = LevelManager::new(3);
        manager.complete_level(0);
        manager.complete_level(1);
        manager.complete_level(2);
        assert!(manager.all_completed());
    }

    #[test]
    fn score_tracker_records_best() {
        let mut tracker = ScoreTracker::new();
        assert!(tracker.record_score(0, 1000));
        assert!(tracker.record_score(0, 2000));
        assert!(!tracker.record_score(0, 500), "Lower score should not replace");
    }

    #[test]
    fn score_tracker_total() {
        let mut tracker = ScoreTracker::new();
        tracker.record_score(0, 1000);
        tracker.record_score(1, 2000);

        assert_eq!(tracker.total_score, 3000);
    }

    #[test]
    fn score_tracker_best_score() {
        let mut tracker = ScoreTracker::new();
        tracker.record_score(0, 1000);
        tracker.record_score(0, 500); // Lower

        assert_eq!(tracker.best_score(0), 1000, "Should keep best score");
    }

    #[test]
    fn game_state_full_playthrough() {
        let mut state = GameState::new();

        // Boot → Title
        assert!(state.transition_to(GameScene::TitleScreen));
        assert_eq!(state.current_scene, GameScene::TitleScreen);

        // Title → LevelSelect
        assert!(state.transition_to(GameScene::LevelSelect));

        // Start level 0
        assert!(state.start_level(0));
        assert_eq!(state.current_scene, GameScene::Playing);

        // Complete level
        state.complete_level(0, 5000);
        assert_eq!(state.score_tracker.best_score(0), 5000);
        assert!(state.level_manager.is_unlocked(1));
    }

    #[test]
    fn game_state_locked_level_fails() {
        let mut state = GameState::new();
        state.current_scene = GameScene::LevelSelect;

        assert!(!state.start_level(5), "Level 5 should be locked");
    }

    #[test]
    fn game_state_victory_after_all_levels() {
        let mut state = GameState::new();
        state.current_scene = GameScene::LevelSelect;
        state.level_manager = LevelManager::new(2);

        state.start_level(0);
        state.complete_level(0, 1000);

        state.current_scene = GameScene::LevelSelect;
        state.start_level(1);
        state.complete_level(1, 2000);

        assert_eq!(state.current_scene, GameScene::Victory);
    }
}
