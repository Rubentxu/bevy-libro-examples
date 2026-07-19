// Capítulo 9. Component Hooks — Bevy 0.19 API
use bevy::ecs::lifecycle::HookContext;
use bevy::ecs::world::DeferredWorld;
use bevy::prelude::*;

#[derive(Component, Debug, Default)]
pub struct Health {
    pub current: u32,
    pub max: u32,
}

#[derive(Component, Debug, Default)]
pub struct Player;

/// Health that auto-clamps on insert via component hook
#[derive(Component, Debug)]
pub struct ClampedHealth {
    pub current: u32,
    pub max: u32,
}

impl Default for ClampedHealth {
    fn default() -> Self {
        Self {
            current: 0,
            max: 100,
        }
    }
}

/// on_add hook: clamps current to max
pub fn clamp_health_on_add(mut world: DeferredWorld, context: HookContext) {
    if let Some(mut health) = world.get_mut::<ClampedHealth>(context.entity) {
        health.current = health.current.min(health.max);
    }
}

pub fn build_hooked_app() -> App {
    let mut app = App::new();
    app.init_resource::<Time>();
    app.world_mut()
        .register_component_hooks::<ClampedHealth>()
        .on_add(clamp_health_on_add);
    app
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn on_add_hook_clamps_health() {
        let mut app = build_hooked_app();
        let entity = app
            .world_mut()
            .spawn(ClampedHealth {
                current: 999,
                max: 100,
            })
            .id();
        assert_eq!(
            app.world().get::<ClampedHealth>(entity).unwrap().current,
            100
        );
    }

    #[test]
    fn hook_fires_on_insert() {
        let mut app = build_hooked_app();
        let entity = app.world_mut().spawn(()).id();
        app.world_mut().entity_mut(entity).insert(ClampedHealth {
            current: 200,
            max: 150,
        });
        assert_eq!(
            app.world().get::<ClampedHealth>(entity).unwrap().current,
            150
        );
    }

    #[test]
    fn hook_respects_valid_values() {
        let mut app = build_hooked_app();
        let entity = app
            .world_mut()
            .spawn(ClampedHealth {
                current: 50,
                max: 100,
            })
            .id();
        assert_eq!(
            app.world().get::<ClampedHealth>(entity).unwrap().current,
            50
        );
    }

    #[test]
    fn world_level_hook_registration() {
        let mut world = World::new();
        world
            .register_component_hooks::<ClampedHealth>()
            .on_add(clamp_health_on_add);
        let entity = world
            .spawn(ClampedHealth {
                current: 500,
                max: 100,
            })
            .id();
        assert_eq!(world.get::<ClampedHealth>(entity).unwrap().current, 100);
    }
}
