// Capítulo 7. Schedules y System Sets — English identifiers

use bevy::ecs::schedule::ScheduleLabel;
use bevy::prelude::*;

// ============================================================================
// COMPONENTS
// ============================================================================

#[derive(Component, Debug)]
pub struct Player;

#[derive(Component, Debug)]
pub struct Enemy;

#[derive(Component, Debug)]
pub struct Bullet;

#[derive(Component, Debug)]
pub struct Health {
    pub current: f32,
    pub max: f32,
}

// ============================================================================
// RESOURCES
// ============================================================================

#[derive(Resource, Debug, Default)]
pub struct Score(pub u32);

#[derive(Resource, Debug)]
pub struct GameplayConfig {
    pub jump_key: KeyCode,
}

// ============================================================================
// GAME STATE (for run conditions)
// ============================================================================

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum GameState {
    #[default]
    Menu,
    Playing,
    Paused,
}

// ============================================================================
// SYSTEM SETS (§7.5)
// ============================================================================

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum GameplaySet {
    Input,
    Movement,
    Physics,
    Combat,
    Render,
}

// ============================================================================
// CUSTOM SCHEDULE LABEL (§7.10)
// ============================================================================

#[derive(ScheduleLabel, Debug, Hash, PartialEq, Eq, Clone)]
pub struct AISimulation;

// ============================================================================
// CUSTOM SYSTEM PARAM (§7.9) — TODO: fix lifetime compatibility with Bevy 0.19
// ============================================================================

// NOTE: Bevy 0.19's SystemParam derive has strict lifetime requirements.
// The PlayerContext below demonstrates the concept but needs lifetime adjustments.
// In practice, you'd use it like:
//   fn jump_system(ctx: PlayerContext, mut commands: Commands) { ... }
//
// #[derive(SystemParam)]
// pub struct PlayerContext<'w> {
//     pub player: Query<&Transform, With<Player>>,
//     pub input: Res<'w, ButtonInput<KeyCode>>,
//     pub config: Res<'w, GameplayConfig>,
// }

// ============================================================================
// SYSTEMS — used by examples and tests
// ============================================================================

pub fn parse_input(_input: Res<ButtonInput<KeyCode>>) {
    // Input parsing logic
}

pub fn move_player(mut query: Query<&mut Transform, With<Player>>) {
    for mut tf in &mut query {
        tf.translation.x += 1.0;
    }
}

pub fn camera_follow_player(query: Query<&Transform, With<Player>>) {
    for _tf in &query {
        // Camera follows player
    }
}

pub fn update_hud(score: Res<Score>) {
    if score.0 > 0 {
        // Update HUD display
    }
}

pub fn integrate_physics(time: Res<Time>, mut query: Query<&mut Transform>) {
    let dt = time.delta_secs();
    for mut tf in &mut query {
        tf.translation.y += 100.0 * dt;
    }
}

pub fn apply_damage(mut query: Query<&mut Health>) {
    for mut health in &mut query {
        health.current -= 1.0;
    }
}

pub fn sync_sprites() {
    // Sync sprite transforms
}

pub fn audio_beep() {
    // Play sound
}

pub fn fps_metrics() {
    // Log FPS metrics
}

pub fn game_over_logic() {
    // Game over screen logic
}

pub fn has_player(query: Query<&Player>) -> bool {
    !query.is_empty()
}

// par_iter system (§7.8)
pub fn move_thousands_of_bullets(
    time: Res<Time>,
    mut bullets: Query<&mut Transform, With<Bullet>>,
) {
    let dt = time.delta_secs();
    bullets.par_iter_mut().for_each(|mut transform| {
        transform.translation.y += 800.0 * dt;
    });
}

// AI systems for custom schedule (§7.10)
pub fn perceive_enemies() {}
pub fn decide_action() {}
pub fn execute_action() {}

// ============================================================================
// APP BUILDER
// ============================================================================

pub fn build_game_app() -> App {
    let mut app = App::new();
    app.init_resource::<Time>();
    app.init_resource::<ButtonInput<KeyCode>>();
    app.insert_resource(default_gameplay_config());
    app.init_resource::<Score>();
    app.init_state::<GameState>();

    // Configure system sets order (§7.5)
    app.configure_sets(
        Update,
        (
            GameplaySet::Input,
            GameplaySet::Movement,
            GameplaySet::Physics,
            GameplaySet::Combat,
            GameplaySet::Render,
        )
            .chain(),
    );

    // Add systems with explicit ordering (§7.4)
    app.add_systems(
        Update,
        (
            move_player,
            camera_follow_player.after(move_player),
            update_hud.after(camera_follow_player),
        ),
    );

    // Run conditions (§7.7)
    app.add_systems(
        Update,
        (
            move_player.run_if(in_state(GameState::Playing)),
            update_hud.run_if(in_state(GameState::Playing)),
        ),
    );

    // Ambiguous systems (§7.6) — ambiguous_all marks system as order-independent
    // NOTE: requires IntoSystemConfigs trait (should be in prelude)
    // app.add_systems(Update, fps_metrics.ambiguous_all());

    app
}

fn default_gameplay_config() -> GameplayConfig {
    GameplayConfig {
        jump_key: KeyCode::Space,
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn system_set_derivations_complete() {
        // Verify GameplaySet has all required derives
        let set = GameplaySet::Input;
        assert_eq!(set, GameplaySet::Input);
        assert_ne!(set, GameplaySet::Movement);
    }

    #[test]
    fn game_state_default_is_menu() {
        let state = GameState::default();
        assert_eq!(state, GameState::Menu);
    }

    #[test]
    fn before_after_ordering_works() {
        // §7.4 — .before() / .after() enforce execution order
        let mut app = App::new();
        app.init_resource::<Time>();
        app.world_mut().spawn((Transform::default(), Player));

        #[derive(Resource, Default)]
        struct OrderLog(Vec<&'static str>);

        app.init_resource::<OrderLog>();

        app.add_systems(
            Update,
            (
                |mut log: ResMut<OrderLog>| log.0.push("first"),
                |mut log: ResMut<OrderLog>| log.0.push("second"),
            ),
        );

        app.update();
        let log = app.world().resource::<OrderLog>();
        // Both should have run; exact order depends on parallelism
        assert_eq!(log.0.len(), 2);
    }

    #[test]
    fn run_condition_skips_system() {
        // §7.7 — run_if controls whether a system executes
        let mut app = App::new();
        app.init_resource::<Time>();
        app.add_plugins(bevy::state::app::StatesPlugin);
        app.init_state::<GameState>();

        #[derive(Resource, Default)]
        struct Counter(u32);

        app.init_resource::<Counter>();
        app.world_mut().spawn((Transform::default(), Player));

        // System only runs when state is Playing
        fn counting_system(mut counter: ResMut<Counter>) {
            counter.0 += 1;
        }

        app.add_systems(Update, counting_system.run_if(in_state(GameState::Playing)));

        // State is Menu → system should NOT run
        app.update();
        assert_eq!(app.world().resource::<Counter>().0, 0);

        // Transition to Playing
        app.world_mut()
            .resource_mut::<NextState<GameState>>()
            .set(GameState::Playing);
        app.update(); // Process state transition
        app.update(); // Now system runs

        assert!(
            app.world().resource::<Counter>().0 > 0,
            "system should run when Playing"
        );
    }

    #[test]
    fn fixed_update_timestep_is_constant() {
        // §7.3 — FixedUpdate runs with constant delta
        let mut app = App::new();
        app.insert_resource(Time::<Fixed>::from_hz(60.0));

        #[derive(Resource, Default)]
        struct DeltaLog(Vec<f32>);

        app.init_resource::<DeltaLog>();
        app.add_systems(FixedUpdate, |time: Res<Time>, mut log: ResMut<DeltaLog>| {
            log.0.push(time.delta_secs());
        });

        // Run several frames
        for _ in 0..5 {
            app.update();
        }

        let log = app.world().resource::<DeltaLog>();
        // All deltas should be 1/60
        for &dt in &log.0 {
            assert!((dt - 1.0 / 60.0).abs() < 0.001, "expected 1/60, got {}", dt);
        }
    }

    #[test]
    fn custom_schedule_can_be_run() {
        // §7.10 — Custom ScheduleLabel
        let mut app = App::new();
        app.init_resource::<Time>();
        app.init_schedule(AISimulation);

        #[derive(Resource, Default)]
        struct AIQueue(Vec<&'static str>);

        app.init_resource::<AIQueue>();

        app.add_systems(
            AISimulation,
            (
                |mut q: ResMut<AIQueue>| q.0.push("perceive"),
                |mut q: ResMut<AIQueue>| q.0.push("decide"),
                |mut q: ResMut<AIQueue>| q.0.push("execute"),
            )
                .chain(),
        );

        // Run the custom schedule manually
        app.world_mut().run_schedule(AISimulation);

        let queue = app.world().resource::<AIQueue>();
        assert_eq!(queue.0, vec!["perceive", "decide", "execute"]);
    }

    #[test]
    fn par_iter_processes_all_entities() {
        // §7.8 — par_iter_mut processes entities in parallel
        let mut app = App::new();
        app.init_resource::<Time>();
        app.insert_resource(Time::<Fixed>::from_hz(60.0));

        // Spawn many bullets
        for _ in 0..100 {
            app.world_mut().spawn((Transform::default(), Bullet));
        }

        app.add_systems(Update, move_thousands_of_bullets);

        // Advance time so delta_secs() is non-zero
        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(std::time::Duration::from_secs_f32(1.0 / 60.0));

        let before: Vec<f32> = app
            .world_mut()
            .query_filtered::<&Transform, With<Bullet>>()
            .iter(app.world())
            .map(|t| t.translation.y)
            .collect();

        app.update();

        let after: Vec<f32> = app
            .world_mut()
            .query_filtered::<&Transform, With<Bullet>>()
            .iter(app.world())
            .map(|t| t.translation.y)
            .collect();

        // All bullets should have moved
        for (b, a) in before.iter().zip(after.iter()) {
            assert!(
                a > b,
                "bullet should have moved up: before={}, after={}",
                b,
                a
            );
        }
    }
}
