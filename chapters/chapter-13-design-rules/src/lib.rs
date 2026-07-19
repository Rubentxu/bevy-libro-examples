// Capítulo 13. Reglas de diseño — Hooks vs Observers vs Systems (Bevy 0.19)
//
// Este capítulo trata sobre CUÁNDO usar cada mecanismo de reacción:
// - Hooks: invariantes de un componente (on_add, on_insert, on_replace, on_remove)
// - Observers: reacciones a eventos/transiciones (N observers por evento)
// - Systems: flujo continuo cada frame
//
// Regla nemotécnica: hooks = invariantes; observers = reacciones; systems = flujo

use bevy::ecs::lifecycle::HookContext;
use bevy::ecs::world::DeferredWorld;
use bevy::prelude::*;

#[derive(Component, Debug)]
pub struct Health {
    pub current: i32,
    pub max: i32,
}

#[derive(Component, Debug, Default)]
pub struct MaxHealthClamped;

/// Hook example: clamp health to [0, max] on insert
pub fn clamp_health_on_insert(mut world: DeferredWorld, context: HookContext) {
    if let Some(mut health) = world.get_mut::<Health>(context.entity) {
        if health.current > health.max {
            health.current = health.max;
        }
        if health.current < 0 {
            health.current = 0;
        }
    }
}

pub fn setup_health_hooks(world: &mut World) {
    world
        .register_component_hooks::<Health>()
        .on_insert(clamp_health_on_insert);
}

/// Observer example: react when any entity dies (health reaches 0)
#[derive(EntityEvent, Debug)]
pub struct DeathEvent {
    entity: Entity,
}

pub fn on_death_observer(trigger: On<DeathEvent>, mut commands: Commands) {
    let event = trigger.event();
    commands.entity(event.entity).despawn();
}

/// System example: continuous flow — check all entities for death each frame
pub fn check_death_system(
    query: Query<(Entity, &Health), Changed<Health>>,
    mut commands: Commands,
) {
    for (entity, health) in &query {
        if health.current <= 0 {
            commands.trigger(DeathEvent { entity });
        }
    }
}

pub fn build_design_rules_app() -> App {
    let mut app = App::new();
    app.init_resource::<Time>();

    // Register hooks
    setup_health_hooks(app.world_mut());

    // Register observer
    app.add_observer(on_death_observer);

    // Register system
    app.add_systems(Update, check_death_system);

    app
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hook_clamps_health_on_insert() {
        let mut world = World::new();
        setup_health_hooks(&mut world);

        // Insert health above max
        let entity = world
            .spawn(Health { current: 200, max: 100 })
            .id();

        let health = world.get::<Health>(entity).unwrap();
        assert_eq!(health.current, 100, "Should be clamped to max");
    }

    #[test]
    fn hook_clamps_negative_health() {
        let mut world = World::new();
        setup_health_hooks(&mut world);

        let entity = world
            .spawn(Health { current: -50, max: 100 })
            .id();

        let health = world.get::<Health>(entity).unwrap();
        assert_eq!(health.current, 0, "Negative health should be clamped to 0");
    }

    #[test]
    fn observer_reacts_to_death_event() {
        let mut app = build_design_rules_app();

        let enemy = app.world_mut().spawn(Health { current: 0, max: 100 }).id();

        // Trigger death event
        app.world_mut().trigger(DeathEvent { entity: enemy });

        // Commands from observers are deferred — need to flush
        app.world_mut().flush();

        // Observer should have despawned the entity
        assert!(
            app.world().get_entity(enemy).is_err(),
            "Entity should be despawned by death observer"
        );
    }

    #[test]
    fn system_detects_death_on_change() {
        let mut app = build_design_rules_app();

        let enemy = app.world_mut().spawn(Health { current: 50, max: 100 }).id();

        // Modify health to 0 (triggers Changed<Health>)
        {
            let mut health = app.world_mut().get_mut::<Health>(enemy).unwrap();
            health.current = 0;
        }

        // Run Update schedule — system should detect death and trigger event
        app.update();

        // Entity should be despawned
        assert!(
            app.world().get_entity(enemy).is_err(),
            "Entity should be despawned after system detects death"
        );
    }

    #[test]
    fn hook_vs_observer_vs_system_semantics() {
        // This test documents the three mechanisms:
        // 1. HOOK: fires on component insertion/replacement (invariant enforcement)
        // 2. OBSERVER: fires on custom event (reaction to discrete transition)
        // 3. SYSTEM: fires every frame on Changed<T> (continuous flow monitoring)

        let mut world = World::new();
        setup_health_hooks(&mut world);

        // HOOK: enforces health invariant at insertion time
        let entity = world.spawn(Health { current: 999, max: 100 }).id();
        let h = world.get::<Health>(entity).unwrap();
        assert_eq!(h.current, 100, "Hook enforced max on insert");

        // OBSERVER and SYSTEM are tested in previous tests
    }

    #[test]
    fn multiple_observers_same_event() {
        // Demonstrates that N observers can react to the same event
        // (unlike hooks, which are one-per-component-per-hook-type)
        use std::sync::atomic::{AtomicUsize, Ordering};
        use std::sync::Arc;

        let counter = Arc::new(AtomicUsize::new(0));
        let c1 = counter.clone();
        let c2 = counter.clone();

        let mut world = World::new();

        fn make_observer(c: Arc<AtomicUsize>) -> impl Fn(On<DeathEvent>) {
            move |_: On<DeathEvent>| {
                c.fetch_add(1, Ordering::SeqCst);
            }
        }

        world.add_observer(make_observer(c1));
        world.add_observer(make_observer(c2));

        let entity = world.spawn(()).id();
        world.trigger(DeathEvent { entity });

        assert_eq!(
            counter.load(Ordering::SeqCst),
            2,
            "Both observers should fire"
        );
    }
}
