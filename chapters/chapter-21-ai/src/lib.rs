// Capítulo 21. AI — FSM, behavior patterns
// Pure logic, no GPU dependency, fully testable.
use std::collections::HashMap;

/// Finite State Machine for game entities
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AIState {
    Idle,
    Patrolling,
    Chasing,
    Attacking,
    Fleeing,
    Dead,
}

/// Conditions that trigger state transitions
#[derive(Clone, Debug)]
pub struct AIContext {
    pub health_pct: f32,
    pub distance_to_target: f32,
    pub can_see_target: bool,
    pub target_in_attack_range: bool,
}

/// Simple FSM that decides next state based on context
pub fn decide_state(current: AIState, ctx: &AIContext) -> AIState {
    if ctx.health_pct <= 0.0 {
        return AIState::Dead;
    }

    match current {
        AIState::Dead => AIState::Dead,
        AIState::Idle => {
            if ctx.can_see_target && ctx.health_pct < 0.3 {
                AIState::Fleeing
            } else if ctx.can_see_target {
                AIState::Chasing
            } else {
                AIState::Patrolling
            }
        }
        AIState::Patrolling => {
            if ctx.can_see_target && ctx.health_pct < 0.3 {
                AIState::Fleeing
            } else if ctx.can_see_target {
                AIState::Chasing
            } else {
                AIState::Patrolling
            }
        }
        AIState::Chasing => {
            if ctx.health_pct < 0.2 {
                AIState::Fleeing
            } else if ctx.target_in_attack_range {
                AIState::Attacking
            } else if !ctx.can_see_target {
                AIState::Idle
            } else {
                AIState::Chasing
            }
        }
        AIState::Attacking => {
            if ctx.health_pct < 0.2 {
                AIState::Fleeing
            } else if !ctx.target_in_attack_range {
                AIState::Chasing
            } else {
                AIState::Attacking
            }
        }
        AIState::Fleeing => {
            if ctx.health_pct > 0.5 {
                AIState::Idle
            } else {
                AIState::Fleeing
            }
        }
    }
}

/// Utility AI: score-based decision making
#[derive(Clone, Debug)]
pub struct Consideration {
    pub name: String,
    pub score: f32,
}

pub fn pick_best_action(considerations: &[Consideration]) -> Option<&str> {
    considerations
        .iter()
        .max_by(|a, b| {
            a.score.partial_cmp(&b.score).unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|c| c.name.as_str())
}

/// Behavior tree node types
#[derive(Clone, Debug)]
pub enum BTNode {
    Action(String),
    Sequence(Vec<BTNode>),
    Selector(Vec<BTNode>),
}

#[derive(Clone, Debug, PartialEq)]
pub enum BTStatus {
    Success,
    Failure,
    Running,
}

pub fn execute_bt(node: &BTNode, ctx: &AIContext) -> BTStatus {
    match node {
        BTNode::Action(name) => {
            // Simulated action execution based on name
            match name.as_str() {
                "attack" => {
                    if ctx.target_in_attack_range {
                        BTStatus::Success
                    } else {
                        BTStatus::Failure
                    }
                }
                "chase" => {
                    if ctx.can_see_target {
                        BTStatus::Running
                    } else {
                        BTStatus::Failure
                    }
                }
                "flee" => {
                    if ctx.health_pct < 0.5 {
                        BTStatus::Running
                    } else {
                        BTStatus::Success
                    }
                }
                "patrol" => BTStatus::Running,
                _ => BTStatus::Failure,
            }
        }
        BTNode::Sequence(children) => {
            for child in children {
                match execute_bt(child, ctx) {
                    BTStatus::Success => continue,
                    other => return other,
                }
            }
            BTStatus::Success
        }
        BTNode::Selector(children) => {
            for child in children {
                match execute_bt(child, ctx) {
                    BTStatus::Failure => continue,
                    other => return other,
                }
            }
            BTStatus::Failure
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx(health: f32, dist: f32, see: bool, attack_range: bool) -> AIContext {
        AIContext {
            health_pct: health,
            distance_to_target: dist,
            can_see_target: see,
            target_in_attack_range: attack_range,
        }
    }

    #[test]
    fn idle_to_chasing_on_see_target() {
        let c = ctx(1.0, 10.0, true, false);
        let next = decide_state(AIState::Idle, &c);
        assert_eq!(next, AIState::Chasing);
    }

    #[test]
    fn chasing_to_attacking_in_range() {
        let c = ctx(1.0, 1.0, true, true);
        let next = decide_state(AIState::Chasing, &c);
        assert_eq!(next, AIState::Attacking);
    }

    #[test]
    fn flee_when_low_health() {
        let c = ctx(0.15, 5.0, true, false);
        let next = decide_state(AIState::Chasing, &c);
        assert_eq!(next, AIState::Fleeing);
    }

    #[test]
    fn dead_is_terminal() {
        let c = ctx(0.0, 0.0, false, false);
        let next = decide_state(AIState::Dead, &c);
        assert_eq!(next, AIState::Dead);
    }

    #[test]
    fn dead_when_health_zero() {
        let c = ctx(0.0, 10.0, true, false);
        let next = decide_state(AIState::Chasing, &c);
        assert_eq!(next, AIState::Dead);
    }

    #[test]
    fn utility_ai_picks_highest_score() {
        let considerations = vec![
            Consideration { name: "attack".to_string(), score: 0.3 },
            Consideration { name: "flee".to_string(), score: 0.8 },
            Consideration { name: "patrol".to_string(), score: 0.5 },
        ];
        let best = pick_best_action(&considerations);
        assert_eq!(best, Some("flee"));
    }

    #[test]
    fn bt_action_attack_in_range() {
        let c = ctx(1.0, 1.0, true, true);
        let node = BTNode::Action("attack".to_string());
        assert_eq!(execute_bt(&node, &c), BTStatus::Success);
    }

    #[test]
    fn bt_action_attack_out_of_range() {
        let c = ctx(1.0, 5.0, true, false);
        let node = BTNode::Action("attack".to_string());
        assert_eq!(execute_bt(&node, &c), BTStatus::Failure);
    }

    #[test]
    fn bt_sequence_all_succeed() {
        let c = ctx(1.0, 1.0, true, true);
        let tree = BTNode::Sequence(vec![
            BTNode::Action("chase".to_string()),
            BTNode::Action("attack".to_string()),
        ]);
        // chase returns Running, so sequence should return Running
        assert_eq!(execute_bt(&tree, &c), BTStatus::Running);
    }

    #[test]
    fn bt_selector_finds_first_success() {
        let c = ctx(1.0, 1.0, true, true);
        let tree = BTNode::Selector(vec![
            BTNode::Action("flee".to_string()),    // Success (health > 0.5)
            BTNode::Action("attack".to_string()),   // Would succeed but not reached
        ]);
        assert_eq!(execute_bt(&tree, &c), BTStatus::Success);
    }

    #[test]
    fn recover_from_fleeing_when_healed() {
        let c = ctx(0.7, 10.0, false, false);
        let next = decide_state(AIState::Fleeing, &c);
        assert_eq!(next, AIState::Idle);
    }

    #[test]
    fn lose_target_returns_to_idle() {
        let c = ctx(1.0, 100.0, false, false);
        let next = decide_state(AIState::Chasing, &c);
        assert_eq!(next, AIState::Idle);
    }
}
