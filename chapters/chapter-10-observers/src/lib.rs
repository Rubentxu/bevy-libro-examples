// Capítulo 10. Observers — Trigger-based events (Bevy 0.19 API)
//
// In Bevy 0.19, the observer system parameter is `On<E>` (renamed from `Trigger<E>`).
// Events are triggered via `world.trigger(event)` or `commands.trigger(event)`.
use bevy::prelude::*;

#[derive(Event, Debug)]
pub struct DamageEvent {
    pub target: Entity,
    pub amount: i32,
}

#[derive(Component, Debug)]
pub struct Health {
    pub current: i32,
    pub max: i32,
}

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct Enemy;

/// Observer that reacts to DamageEvent and reduces Health.
/// In Bevy 0.19, the first parameter is `On<E>`, not `Trigger<E>`.
pub fn on_damage(trigger: On<DamageEvent>, mut query: Query<&mut Health>) {
    let event = trigger.event();
    if let Ok(mut health) = query.get_mut(event.target) {
        health.current -= event.amount;
        println!(
            "Entity {:?} took {} damage, HP now {}",
            event.target, event.amount, health.current
        );
    }
}

pub fn build_observer_app() -> App {
    let mut app = App::new();
    app.init_resource::<Time>();
    app.add_observer(on_damage);

    let player = app
        .world_mut()
        .spawn((Health { current: 100, max: 100 }, Player))
        .id();
    let enemy = app
        .world_mut()
        .spawn((Health { current: 50, max: 50 }, Enemy))
        .id();

    app.insert_resource(PlayerEntity(player));
    app.insert_resource(EnemyEntity(enemy));
    app
}

#[derive(Resource)]
pub struct PlayerEntity(pub Entity);

#[derive(Resource)]
pub struct EnemyEntity(pub Entity);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn observer_reacts_to_trigger() {
        let mut app = build_observer_app();
        let enemy = app.world().resource::<EnemyEntity>().0;

        // Trigger the event directly on the world
        app.world_mut()
            .trigger(DamageEvent { target: enemy, amount: 30 });

        let health = app.world().get::<Health>(enemy).unwrap();
        assert_eq!(health.current, 20, "50 - 30 = 20");
    }

    #[test]
    fn observer_can_trigger_from_system() {
        let mut app = build_observer_app();
        let enemy = app.world().resource::<EnemyEntity>().0;

        app.add_systems(Update, move |mut commands: Commands| {
            commands.trigger(DamageEvent { target: enemy, amount: 10 });
        });
        app.update();

        let health = app.world().get::<Health>(enemy).unwrap();
        assert_eq!(health.current, 40, "50 - 10 = 40");
    }

    #[test]
    fn multiple_triggers_accumulate() {
        let mut app = build_observer_app();
        let enemy = app.world().resource::<EnemyEntity>().0;

        app.world_mut()
            .trigger(DamageEvent { target: enemy, amount: 20 });
        app.world_mut()
            .trigger(DamageEvent { target: enemy, amount: 10 });

        let health = app.world().get::<Health>(enemy).unwrap();
        assert_eq!(health.current, 20, "50 - 20 - 10 = 20");
    }

    #[test]
    fn observer_damages_player() {
        let mut app = build_observer_app();
        let player = app.world().resource::<PlayerEntity>().0;

        app.world_mut()
            .trigger(DamageEvent { target: player, amount: 25 });

        let health = app.world().get::<Health>(player).unwrap();
        assert_eq!(health.current, 75, "100 - 25 = 75");
    }
}
