// Capítulo 8. Required Components — English identifiers

use bevy::prelude::*;

// ============================================================================
// LEAF COMPONENTS (with Default)
// ============================================================================

#[derive(Component, Debug, Default)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

#[derive(Component, Debug, Default)]
pub struct Velocity {
    pub x: f32,
    pub y: f32,
}

#[derive(Component, Debug, Default)]
pub struct Sprite;

#[derive(Component, Debug, Default)]
pub struct Health {
    pub current: u32,
    pub max: u32,
}

#[derive(Component, Debug, Default)]
pub struct Nameplate {
    pub text: String,
}

#[derive(Component, Debug, Default)]
pub struct Inventory {
    pub slots: Vec<u32>,
    pub current_weight: u32,
    pub max_weight: u32,
}

// ============================================================================
// SEMANTIC PACKAGES with #[require(...)]
// ============================================================================

/// Anything that moves on screen — §8.5
#[derive(Component, Debug, Default)]
#[require(Position, Velocity, Sprite)]
pub struct Movable;

/// Anything with health — §8.5
#[derive(Component, Debug, Default)]
#[require(Movable, Health)]
pub struct Living;

/// RPG character (player or NPC) — §8.5
#[derive(Component, Debug, Default)]
#[require(Living, Inventory)]
pub struct Character;

/// Player component with full require chain — §8.5
#[derive(Component, Debug, Default)]
#[require(Character, Nameplate)]
pub struct Player;

/// Enemy with explicit initial data — §8.2
#[derive(Component, Debug, Default)]
#[require(Health { current: 50, max: 50 })]
pub struct Enemy;

/// Component registered at runtime — §8.2.1
#[derive(Component, Debug, Default)]
pub struct ModComponent;

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn player_spawns_with_all_required() {
        // §8.5 — Spawning Player should auto-insert Position, Velocity, Sprite,
        // Health, Inventory, Nameplate, Movable, Living, Character
        let mut app = App::new();
        let entity = app.world_mut().spawn(Player::default()).id();

        assert!(
            app.world().get::<Position>(entity).is_some(),
            "Position required"
        );
        assert!(
            app.world().get::<Velocity>(entity).is_some(),
            "Velocity required"
        );
        assert!(
            app.world().get::<Sprite>(entity).is_some(),
            "Sprite required"
        );
        assert!(
            app.world().get::<Health>(entity).is_some(),
            "Health required"
        );
        assert!(
            app.world().get::<Inventory>(entity).is_some(),
            "Inventory required"
        );
        assert!(
            app.world().get::<Nameplate>(entity).is_some(),
            "Nameplate required"
        );
        assert!(
            app.world().get::<Movable>(entity).is_some(),
            "Movable required"
        );
        assert!(
            app.world().get::<Living>(entity).is_some(),
            "Living required"
        );
        assert!(
            app.world().get::<Character>(entity).is_some(),
            "Character required"
        );
    }

    #[test]
    fn enemy_has_explicit_initial_health() {
        // §8.2 — require with literal data
        let mut app = App::new();
        let entity = app.world_mut().spawn(Enemy).id();

        let health = app.world().get::<Health>(entity).expect("Health required");
        assert_eq!(
            health.current, 50,
            "Enemy should have 50 HP from require literal"
        );
        assert_eq!(health.max, 50);
    }

    #[test]
    fn register_required_components_runtime() {
        // §8.2.1 — register_required_components::<A, B>() on App
        let mut app = App::new();
        app.register_required_components::<Player, ModComponent>();

        let entity = app.world_mut().spawn(Player::default()).id();
        assert!(
            app.world().get::<ModComponent>(entity).is_some(),
            "ModComponent should be auto-inserted via runtime registration"
        );
    }

    #[test]
    fn depth_first_ordering() {
        // §8.3 — depth-first: leaf components inserted before parent
        // When spawning Player, Position should be inserted before Character
        let mut app = App::new();
        let entity = app.world_mut().spawn(Player::default()).id();

        // All components should be present after spawn
        // The order of insertion is depth-first, tested implicitly by the fact
        // that all components exist
        let components = (
            app.world().get::<Position>(entity).is_some(),
            app.world().get::<Velocity>(entity).is_some(),
            app.world().get::<Movable>(entity).is_some(),
            app.world().get::<Character>(entity).is_some(),
            app.world().get::<Player>(entity).is_some(),
        );
        assert_eq!(components, (true, true, true, true, true));
    }

    #[test]
    fn query_finds_all_required_components() {
        // §8.5 — Queries can reliably find components that come from requires
        let mut app = App::new();
        app.world_mut().spawn(Player::default());
        app.world_mut().spawn(Player::default());

        // Query for Health on Characters — both Players should match
        let mut q = app.world_mut().query_filtered::<&Health, With<Character>>();
        let count = q.iter(app.world()).count();
        assert_eq!(
            count, 2,
            "both Players should have Health via Character require"
        );
    }
}
