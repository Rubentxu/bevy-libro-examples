// Capítulo 11. Relationships — Bubbling, run conditions, custom relations (Bevy 0.19)
use bevy::prelude::*;
use bevy::ecs::hierarchy::ChildOf;

#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct Player;

#[derive(Component)]
pub struct Enemy;

#[derive(Component, Debug)]
pub struct Health {
    pub current: i32,
    pub max: i32,
}

// === Custom relationship: Likes / LikedBy ===

#[derive(Component, Debug)]
#[relationship(relationship_target = LikedBy)]
pub struct Likes(#[entities] pub Entity);

#[derive(Component, Debug)]
#[relationship_target(relationship = Likes)]
pub struct LikedBy(Vec<Entity>);

// === Event for bubbling ===

#[derive(EntityEvent, Debug)]
pub struct DamageEvent {
    entity: Entity,
    pub amount: i32,
}

/// Observer that reacts to DamageEvent and reduces Health.
/// In a real game, this would bubble up to the parent (e.g., squad leader).
pub fn on_damage(mut trigger: On<DamageEvent>, mut query: Query<&mut Health>) {
    let event = trigger.event();
    if let Ok(mut health) = query.get_mut(event.entity) {
        health.current -= event.amount;
        if health.current < 0 {
            health.current = 0;
        }
    }
}

pub fn build_relationships_app() -> App {
    let mut app = App::new();
    app.init_resource::<Time>();
    app.add_observer(on_damage);
    app
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parent_child_hierarchy() {
        let mut world = World::new();

        let parent = world.spawn(Health { current: 100, max: 100 }).id();
        let child1 = world.spawn((ChildOf(parent), Health { current: 50, max: 50 })).id();
        let child2 = world.spawn((ChildOf(parent), Health { current: 30, max: 30 })).id();

        // Verify parent has children
        let children = world.get::<bevy::ecs::hierarchy::Children>(parent);
        assert!(children.is_some(), "Parent should have Children component");

        let children_ids: Vec<Entity> = children.unwrap().iter().collect();
        assert!(children_ids.contains(&child1), "child1 should be in parent's children");
        assert!(children_ids.contains(&child2), "child2 should be in parent's children");
    }

    #[test]
    fn childof_parent_method() {
        let mut world = World::new();
        let parent = world.spawn(()).id();
        let child = world.spawn(ChildOf(parent)).id();

        let child_of = world.get::<ChildOf>(child).unwrap();
        assert_eq!(child_of.parent(), parent);
    }

    #[test]
    fn custom_relationship_likes() {
        let mut world = World::new();

        let alice = world.spawn(()).id();
        let bob = world.spawn(()).id();

        // Alice likes Bob — this auto-inserts LikedBy(alice) on Bob
        world.entity_mut(alice).insert(Likes(bob));

        // Bob should have LikedBy with Alice
        let liked_by_bob = world.get::<LikedBy>(bob);
        assert!(liked_by_bob.is_some(), "Bob should have LikedBy component");

        let liked_by = liked_by_bob.unwrap();
        assert!(liked_by.0.contains(&alice), "Bob should be liked by Alice");
    }

    #[test]
    fn despawn_parent_removes_children() {
        let mut world = World::new();

        let parent = world.spawn(()).id();
        let child = world.spawn(ChildOf(parent)).id();
        let grandchild = world.spawn(ChildOf(child)).id();

        // Despawning parent should despawn children recursively
        world.entity_mut(parent).despawn();

        assert!(world.get_entity(parent).is_err(), "Parent should be despawned");
        assert!(world.get_entity(child).is_err(), "Child should be despawned");
        assert!(world.get_entity(grandchild).is_err(), "Grandchild should be despawned");
    }

    #[test]
    fn observer_triggers_on_entity_event() {
        let mut app = build_relationships_app();

        let enemy = app.world_mut().spawn((Health { current: 50, max: 50 }, Enemy)).id();

        app.world_mut().trigger(DamageEvent {
            entity: enemy,
            amount: 20,
        });

        let health = app.world().get::<Health>(enemy).unwrap();
        assert_eq!(health.current, 30, "50 - 20 = 30");
    }

    #[test]
    fn observer_clamps_health_to_zero() {
        let mut app = build_relationships_app();

        let enemy = app.world_mut().spawn((Health { current: 10, max: 10 }, Enemy)).id();

        app.world_mut().trigger(DamageEvent {
            entity: enemy,
            amount: 50,
        });

        let health = app.world().get::<Health>(enemy).unwrap();
        assert_eq!(health.current, 0, "Health should be clamped to 0");
    }

    #[test]
    fn query_descendants() {
        let mut world = World::new();

        let root = world.spawn(()).id();
        let child_a = world.spawn(ChildOf(root)).id();
        let child_b = world.spawn(ChildOf(root)).id();
        let grandchild = world.spawn(ChildOf(child_a)).id();

        // Query all ChildOf components
        let mut query = world.query::<&ChildOf>();
        let parents: Vec<Entity> = query.iter(&world).map(|c| c.parent()).collect();

        assert!(parents.contains(&root));
        assert!(parents.contains(&child_a));
    }
}
