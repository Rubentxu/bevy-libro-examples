// Capítulo 14D. Conditional Systems — State machines, run conditions
use bevy::prelude::*;

/// Game state enum
#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GameState {
    #[default]
    Loading,
    MainMenu,
    Playing,
    Paused,
    GameOver,
}

/// Condition: is the game in an active gameplay state?
pub fn in_gameplay(state: &State<GameState>) -> bool {
    matches!(**state, GameState::Playing)
}

/// Condition: is the game paused?
pub fn is_paused(state: &State<GameState>) -> bool {
    matches!(**state, GameState::Paused)
}

/// Condition: is the game over?
pub fn is_game_over(state: &State<GameState>) -> bool {
    matches!(**state, GameState::GameOver)
}

/// Simple state machine for entities (not Bevy States, but a pattern)
#[derive(Clone, Debug, PartialEq)]
pub enum EntityState {
    Idle,
    Moving,
    Attacking,
    Stunned,
    Dead,
}

pub struct StateMachine {
    pub current: EntityState,
    pub time_in_state: f32,
}

impl StateMachine {
    pub fn new() -> Self {
        Self {
            current: EntityState::Idle,
            time_in_state: 0.0,
        }
    }

    pub fn transition(&mut self, new_state: EntityState) -> bool {
        if self.can_transition(&new_state) {
            self.current = new_state;
            self.time_in_state = 0.0;
            true
        } else {
            false
        }
    }

    fn can_transition(&self, target: &EntityState) -> bool {
        use EntityState::*;
        match (&self.current, target) {
            // Can't transition from Dead
            (Dead, _) => false,
            // Can always go to Dead
            (_, Dead) => true,
            // Can't attack while stunned
            (Stunned, Attacking) => false,
            // Everything else is allowed
            _ => true,
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.time_in_state += dt;

        // Auto-transitions
        match self.current {
            EntityState::Attacking if self.time_in_state > 0.5 => {
                self.transition(EntityState::Idle);
            }
            EntityState::Stunned if self.time_in_state > 1.0 => {
                self.transition(EntityState::Idle);
            }
            _ => {}
        }
    }
}

impl Default for StateMachine {
    fn default() -> Self { Self::new() }
}

/// Scene transition manager
pub struct SceneManager {
    pub current_state: GameState,
    pub pending_state: Option<GameState>,
    pub transition_time: f32,
    pub transition_duration: f32,
}

impl SceneManager {
    pub fn new() -> Self {
        Self {
            current_state: GameState::Loading,
            pending_state: None,
            transition_time: 0.0,
            transition_duration: 0.5,
        }
    }

    pub fn request_transition(&mut self, target: GameState) {
        if self.pending_state.is_none() {
            self.pending_state = Some(target);
            self.transition_time = 0.0;
        }
    }

    pub fn update(&mut self, dt: f32) -> bool {
        if let Some(target) = self.pending_state {
            self.transition_time += dt;
            if self.transition_time >= self.transition_duration {
                self.current_state = target;
                self.pending_state = None;
                return true; // Transition complete
            }
        }
        false
    }

    pub fn is_transitioning(&self) -> bool {
        self.pending_state.is_some()
    }
}

impl Default for SceneManager {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn state_machine_starts_idle() {
        let sm = StateMachine::new();
        assert_eq!(sm.current, EntityState::Idle);
    }

    #[test]
    fn state_machine_valid_transition() {
        let mut sm = StateMachine::new();
        assert!(sm.transition(EntityState::Moving));
        assert_eq!(sm.current, EntityState::Moving);
    }

    #[test]
    fn state_machine_dead_is_terminal() {
        let mut sm = StateMachine::new();
        sm.transition(EntityState::Dead);
        assert!(!sm.transition(EntityState::Idle), "Cannot leave Dead state");
    }

    #[test]
    fn state_machine_stunned_cannot_attack() {
        let mut sm = StateMachine::new();
        sm.transition(EntityState::Stunned);
        assert!(!sm.transition(EntityState::Attacking), "Cannot attack while stunned");
    }

    #[test]
    fn state_machine_attack_auto_ends() {
        let mut sm = StateMachine::new();
        sm.transition(EntityState::Attacking);

        sm.update(0.6); // Past 0.5s attack duration
        assert_eq!(sm.current, EntityState::Idle, "Attack should auto-end");
    }

    #[test]
    fn state_machine_stun_auto_ends() {
        let mut sm = StateMachine::new();
        sm.transition(EntityState::Stunned);

        sm.update(1.1); // Past 1.0s stun duration
        assert_eq!(sm.current, EntityState::Idle, "Stun should auto-end");
    }

    #[test]
    fn state_machine_time_accumulates() {
        let mut sm = StateMachine::new();
        sm.update(0.5);
        assert!((sm.time_in_state - 0.5).abs() < 0.001);
    }

    #[test]
    fn scene_manager_request_transition() {
        let mut manager = SceneManager::new();
        assert!(!manager.is_transitioning());

        manager.request_transition(GameState::Playing);
        assert!(manager.is_transitioning());
    }

    #[test]
    fn scene_manager_transition_completes() {
        let mut manager = SceneManager::new();
        manager.request_transition(GameState::MainMenu);

        // Before duration
        assert!(!manager.update(0.3));
        assert!(manager.is_transitioning());

        // After duration
        assert!(manager.update(0.3));
        assert!(!manager.is_transitioning());
        assert_eq!(manager.current_state, GameState::MainMenu);
    }

    #[test]
    fn scene_manager_no_double_transition() {
        let mut manager = SceneManager::new();
        manager.request_transition(GameState::Playing);
        manager.request_transition(GameState::Paused); // Should be ignored

        assert_eq!(manager.pending_state, Some(GameState::Playing));
    }

    #[test]
    fn game_state_default() {
        assert_eq!(GameState::default(), GameState::Loading);
    }
}
