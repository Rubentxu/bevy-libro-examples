// Capítulo 14A. Plugins — Registration pattern, modularity
use bevy::prelude::*;
use bevy::app::PluginGroupBuilder;

/// A plugin that tracks its own registration
#[derive(Component, Debug, Default)]
pub struct PluginMarker(pub &'static str);

/// Combat plugin: registers combat systems and resources
pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CombatStats>();
        app.add_systems(Update, combat_tick);
    }
}

#[derive(Resource, Debug, Default)]
pub struct CombatStats {
    pub total_attacks: u32,
    pub total_damage: u32,
}

#[derive(Component, Debug)]
pub struct Health { pub current: i32, pub max: i32 }

#[derive(Component, Debug)]
pub struct AttackPower(pub i32);

fn combat_tick(mut stats: ResMut<CombatStats>, query: Query<&AttackPower>) {
    for power in &query {
        stats.total_attacks += 1;
        stats.total_damage += power.0 as u32;
    }
}

/// Plugin that depends on another plugin
pub struct DifficultyPlugin;

impl Plugin for DifficultyPlugin {
    fn build(&self, app: &mut App) {
        // Ensure CombatPlugin is registered first
        if !app.is_plugin_added::<CombatPlugin>() {
            app.add_plugins(CombatPlugin);
        }
        app.init_resource::<DifficultySettings>();
    }
}

#[derive(Resource, Debug, Clone, Copy)]
pub struct DifficultySettings {
    pub enemy_health_mult: f32,
    pub enemy_damage_mult: f32,
}

impl Default for DifficultySettings {
    fn default() -> Self {
        Self { enemy_health_mult: 1.0, enemy_damage_mult: 1.0 }
    }
}

/// Plugin group: bundles multiple plugins together
pub struct GameCorePluginGroup;

impl PluginGroup for GameCorePluginGroup {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(CombatPlugin)
            .add(DifficultyPlugin)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plugin_registers_resource() {
        let mut app = App::new();
        app.add_plugins(CombatPlugin);

        assert!(app.world().contains_resource::<CombatStats>());
    }

    #[test]
    fn plugin_system_runs() {
        let mut app = App::new();
        app.add_plugins(CombatPlugin);

        // Spawn entity with AttackPower
        app.world_mut().spawn(AttackPower(25));

        app.update();

        let stats = app.world().resource::<CombatStats>();
        assert_eq!(stats.total_attacks, 1);
        assert_eq!(stats.total_damage, 25);
    }

    #[test]
    fn plugin_dependency_auto_registers() {
        let mut app = App::new();
        app.add_plugins(DifficultyPlugin);

        // CombatPlugin should also be registered
        assert!(app.is_plugin_added::<CombatPlugin>());
        assert!(app.world().contains_resource::<DifficultySettings>());
    }

    #[test]
    fn plugin_group_bundles_all() {
        let mut app = App::new();
        app.add_plugins(GameCorePluginGroup);

        assert!(app.is_plugin_added::<CombatPlugin>());
        assert!(app.is_plugin_added::<DifficultyPlugin>());
    }

    #[test]
    fn difficulty_settings_default() {
        let settings = DifficultySettings::default();
        assert_eq!(settings.enemy_health_mult, 1.0);
        assert_eq!(settings.enemy_damage_mult, 1.0);
    }

    #[test]
    fn plugin_idempotent() {
        let mut app = App::new();
        app.add_plugins(CombatPlugin);

        // Adding again should not panic (is_plugin_added check)
        if !app.is_plugin_added::<CombatPlugin>() {
            app.add_plugins(CombatPlugin);
        }

        let stats = app.world().resource::<CombatStats>();
        assert_eq!(stats.total_attacks, 0); // No entities to process
    }
}
