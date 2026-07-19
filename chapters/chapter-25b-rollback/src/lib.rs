// Capítulo 25B. Rollback Netcode — State snapshots, input delay, re-simulation
use std::collections::VecDeque;

/// A single frame's input from a player
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Input {
    pub left: bool,
    pub right: bool,
    pub up: bool,
    pub down: bool,
    pub action: bool,
}

impl Input {
    pub fn empty() -> Self {
        Self::default()
    }

    pub fn from_bits(left: bool, right: bool, up: bool, down: bool, action: bool) -> Self {
        Self { left, right, up, down, action }
    }
}

/// Game state snapshot that can be saved and restored
#[derive(Clone, Debug, PartialEq)]
pub struct GameState {
    pub tick: u64,
    pub positions: Vec<(u32, f32, f32)>, // (entity_id, x, y)
    pub healths: Vec<(u32, i32)>,       // (entity_id, hp)
}

impl GameState {
    pub fn new(tick: u64) -> Self {
        Self {
            tick,
            positions: Vec::new(),
            healths: Vec::new(),
        }
    }

    pub fn snapshot(&self) -> Self {
        self.clone()
    }

    pub fn restore(&mut self, other: &GameState) {
        self.tick = other.tick;
        self.positions = other.positions.clone();
        self.healths = other.healths.clone();
    }
}

/// Ring buffer for storing recent states (for rollback)
pub struct StateBuffer {
    states: VecDeque<(u64, GameState)>,
    max_size: usize,
}

impl StateBuffer {
    pub fn new(max_size: usize) -> Self {
        Self {
            states: VecDeque::with_capacity(max_size),
            max_size,
        }
    }

    pub fn push(&mut self, tick: u64, state: GameState) {
        if self.states.len() >= self.max_size {
            self.states.pop_front();
        }
        self.states.push_back((tick, state));
    }

    pub fn get(&self, tick: u64) -> Option<&GameState> {
        self.states
            .iter()
            .find(|(t, _)| *t == tick)
            .map(|(_, s)| s)
    }

    pub fn oldest_tick(&self) -> Option<u64> {
        self.states.front().map(|(t, _)| *t)
    }

    pub fn newest_tick(&self) -> Option<u64> {
        self.states.back().map(|(t, _)| *t)
    }

    pub fn len(&self) -> usize {
        self.states.len()
    }
}

/// Input buffer per player (stores confirmed + predicted inputs)
pub struct InputBuffer {
    inputs: VecDeque<(u64, Input)>,
    max_delay: usize,
}

impl InputBuffer {
    pub fn new(max_delay: usize) -> Self {
        Self {
            inputs: VecDeque::with_capacity(max_delay),
            max_delay,
        }
    }

    pub fn add_input(&mut self, tick: u64, input: Input) {
        self.inputs.push_back((tick, input));
        while self.inputs.len() > self.max_delay {
            self.inputs.pop_front();
        }
    }

    pub fn get_input(&self, tick: u64) -> Input {
        self.inputs
            .iter()
            .find(|(t, _)| *t == tick)
            .map(|(_, i)| *i)
            .unwrap_or_default()
    }

    /// Check if we have confirmed input for a given tick
    pub fn has_confirmed(&self, tick: u64) -> bool {
        self.inputs.iter().any(|(t, _)| *t == tick)
    }
}

/// Rollback manager: coordinates state saves, input correction, and re-simulation
pub struct RollbackManager {
    pub current_tick: u64,
    pub state_buffer: StateBuffer,
    pub confirmed_tick: u64,
    pub max_rollback: usize,
}

impl RollbackManager {
    pub fn new(max_rollback: usize) -> Self {
        Self {
            current_tick: 0,
            state_buffer: StateBuffer::new(max_rollback),
            confirmed_tick: 0,
            max_rollback,
        }
    }

    /// Save a state snapshot for the current tick
    pub fn save_state(&mut self, state: &GameState) {
        self.state_buffer.push(state.tick, state.snapshot());
    }

    /// Attempt rollback to a specific tick
    /// Returns the state to restore, or None if tick is too old
    pub fn rollback_to(&self, tick: u64) -> Option<&GameState> {
        if tick < self.confirmed_tick.saturating_sub(self.max_rollback as u64) {
            return None; // Too far back
        }
        self.state_buffer.get(tick)
    }

    /// Advance simulation tick
    pub fn advance(&mut self) {
        self.current_tick += 1;
    }
}

/// Simulate one step of game logic (simplified)
pub fn simulate_step(state: &mut GameState, input: Input) {
    state.tick += 1;

    // Move player 0 based on input
    if let Some(entry) = state.positions.get_mut(0) {
        let speed = 5.0;
        if input.left { entry.1 -= speed; }
        if input.right { entry.1 += speed; }
        if input.up { entry.2 += speed; }
        if input.down { entry.2 -= speed; }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn state_snapshot_and_restore() {
        let mut state = GameState::new(10);
        state.positions.push((1, 100.0, 200.0));
        state.healths.push((1, 50));

        let snapshot = state.snapshot();

        // Modify state
        state.positions[0].1 = 999.0;
        state.tick = 99;

        // Restore
        state.restore(&snapshot);

        assert_eq!(state.tick, 10);
        assert_eq!(state.positions[0].1, 100.0);
    }

    #[test]
    fn state_buffer_stores_and_retrieves() {
        let mut buffer = StateBuffer::new(10);

        for tick in 0..5 {
            buffer.push(tick, GameState::new(tick));
        }

        assert_eq!(buffer.len(), 5);
        assert_eq!(buffer.oldest_tick(), Some(0));
        assert_eq!(buffer.newest_tick(), Some(4));

        let state = buffer.get(2);
        assert!(state.is_some());
        assert_eq!(state.unwrap().tick, 2);
    }

    #[test]
    fn state_buffer_evicts_oldest() {
        let mut buffer = StateBuffer::new(3);

        for tick in 0..5 {
            buffer.push(tick, GameState::new(tick));
        }

        assert_eq!(buffer.len(), 3);
        assert_eq!(buffer.oldest_tick(), Some(2), "Should evict ticks 0 and 1");
        assert_eq!(buffer.newest_tick(), Some(4));
    }

    #[test]
    fn input_buffer_stores_inputs() {
        let mut buf = InputBuffer::new(10);
        buf.add_input(0, Input::from_bits(true, false, false, false, false));
        buf.add_input(1, Input::from_bits(false, true, false, false, false));

        let input0 = buf.get_input(0);
        assert!(input0.left);
        assert!(!input0.right);

        let input1 = buf.get_input(1);
        assert!(!input1.left);
        assert!(input1.right);
    }

    #[test]
    fn input_buffer_missing_tick_returns_default() {
        let buf = InputBuffer::new(10);
        let input = buf.get_input(999);
        assert_eq!(input, Input::empty());
    }

    #[test]
    fn input_buffer_has_confirmed() {
        let mut buf = InputBuffer::new(10);
        buf.add_input(5, Input::empty());

        assert!(buf.has_confirmed(5));
        assert!(!buf.has_confirmed(6));
    }

    #[test]
    fn rollback_manager_saves_and_restores() {
        let mut manager = RollbackManager::new(10);

        let state = GameState::new(5);
        manager.save_state(&state);

        let restored = manager.rollback_to(5);
        assert!(restored.is_some());
        assert_eq!(restored.unwrap().tick, 5);
    }

    #[test]
    fn rollback_too_far_returns_none() {
        let manager = RollbackManager::new(5);

        // Try to rollback without any saved state
        let result = manager.rollback_to(0);
        assert!(result.is_none());
    }

    #[test]
    fn simulate_step_advances_tick() {
        let mut state = GameState::new(10);
        state.positions.push((0, 0.0, 0.0));

        simulate_step(&mut state, Input::empty());
        assert_eq!(state.tick, 11);
    }

    #[test]
    fn simulate_step_moves_player() {
        let mut state = GameState::new(0);
        state.positions.push((0, 100.0, 100.0));

        // Move right
        simulate_step(&mut state, Input::from_bits(false, true, false, false, false));
        assert!(state.positions[0].1 > 100.0, "Player should move right");

        // Move up
        simulate_step(&mut state, Input::from_bits(false, false, true, false, false));
        assert!(state.positions[0].2 > 100.0, "Player should move up");
    }

    #[test]
    fn rollback_resimulation_corrects_state() {
        let mut state = GameState::new(0);
        state.positions.push((0, 0.0, 0.0));

        // Simulate 3 ticks with wrong input (predicted)
        for _ in 0..3 {
            simulate_step(&mut state, Input::from_bits(false, true, false, false, false));
        }

        // Player should be at x = 15 (3 ticks * 5 speed)
        let wrong_x = state.positions[0].1;
        assert!((wrong_x - 15.0).abs() < 0.001);

        // Now re-simulate from tick 0 with correct input (left instead of right)
        state.positions[0] = (0, 0.0, 0.0);
        state.tick = 0;
        for _ in 0..3 {
            simulate_step(&mut state, Input::from_bits(true, false, false, false, false));
        }

        // Player should now be at x = -15
        let correct_x = state.positions[0].1;
        assert!((correct_x - (-15.0)).abs() < 0.001);
    }
}
