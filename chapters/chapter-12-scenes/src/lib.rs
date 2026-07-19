// Capítulo 12. Escenas — bsn! y DynamicScene (Bevy 0.19)
//
// bsn! (Bevy Scene Notation) es una macro que produce estructuras declarativas.
// Aquí testeamos los conceptos subyacentes: jerarquías, required components,
// y construcción programática de escenas que bsn! automatiza.
use bevy::prelude::*;
use bevy::ecs::hierarchy::ChildOf;

#[derive(Component, Debug, Clone)]
pub struct Position(pub Vec2);

#[derive(Component, Debug, Clone)]
pub struct Name(pub String);

#[derive(Component, Debug, Clone)]
pub struct Enemy;

#[derive(Component, Debug, Clone)]
pub struct Health {
    pub current: i32,
    pub max: i32,
}

/// Required component: anything with Health must have a MaxHealth marker
#[derive(Component, Debug, Clone, Default)]
pub struct CombatTag;

#[derive(Component, Debug, Clone)]
#[require(CombatTag)]
pub struct Warrior {
    pub strength: i32,
}

/// Build a scene programmatically — this is what bsn! automates.
pub fn build_village_scene(world: &mut World) -> Vec<Entity> {
    let mut entities = Vec::new();

    // Village root
    let village = world.spawn((
        Name("Village".to_string()),
        Position(Vec2::ZERO),
    )).id();

    // House 1 with inhabitants
    let house1 = world.spawn((
        Name("House".to_string()),
        Position(Vec2::new(10.0, 0.0)),
        ChildOf(village),
    )).id();

    let npc1 = world.spawn((
        Name("Blacksmith".to_string()),
        Position(Vec2::new(11.0, 0.5)),
        Health { current: 100, max: 100 },
        Warrior { strength: 15 },
        ChildOf(house1),
    )).id();

    let npc2 = world.spawn((
        Name("Apprentice".to_string()),
        Position(Vec2::new(10.5, 0.8)),
        Health { current: 80, max: 80 },
        ChildOf(house1),
    )).id();

    // House 2 with enemies
    let house2 = world.spawn((
        Name("Ruined House".to_string()),
        Position(Vec2::new(-10.0, 5.0)),
        ChildOf(village),
    )).id();

    let enemy = world.spawn((
        Name("Goblin".to_string()),
        Enemy,
        Health { current: 30, max: 30 },
        Position(Vec2::new(-9.5, 5.2)),
        ChildOf(house2),
    )).id();

    entities.extend([village, house1, npc1, npc2, house2, enemy]);
    entities
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scene_builds_hierarchy() {
        let mut world = World::new();
        let entities = build_village_scene(&mut world);

        assert_eq!(entities.len(), 6, "Should have 6 entities");

        let village = entities[0];
        let house1 = entities[1];
        let npc1 = entities[2];

        // Village should have children
        let children = world.get::<bevy::ecs::hierarchy::Children>(village);
        assert!(children.is_some(), "Village should have Children");

        // NPC1 should be child of house1
        let child_of = world.get::<ChildOf>(npc1).unwrap();
        assert_eq!(child_of.parent(), house1);
    }

    #[test]
    fn required_component_auto_inserted() {
        let mut world = World::new();

        // Warrior requires CombatTag
        let warrior = world.spawn(Warrior { strength: 10 }).id();

        assert!(
            world.get::<CombatTag>(warrior).is_some(),
            "CombatTag should be auto-inserted by #[require]"
        );
    }

    #[test]
    fn scene_has_expected_components() {
        let mut world = World::new();
        let entities = build_village_scene(&mut world);

        let enemy = entities[5];
        assert!(world.get::<Enemy>(enemy).is_some(), "Enemy should have Enemy component");
        assert!(world.get::<Health>(enemy).is_some(), "Enemy should have Health");

        let health = world.get::<Health>(enemy).unwrap();
        assert_eq!(health.current, 30);
        assert_eq!(health.max, 30);
    }

    #[test]
    fn despawn_village_cascades() {
        let mut world = World::new();
        let entities = build_village_scene(&mut world);

        let village = entities[0];
        let npc1 = entities[2];

        // Despawning village should cascade to all descendants
        world.entity_mut(village).despawn();

        assert!(world.get_entity(village).is_err());
        assert!(world.get_entity(npc1).is_err(), "NPC should be despawned with village");
    }

    #[test]
    fn warrior_has_combat_tag_and_strength() {
        let mut world = World::new();
        let warrior = world.spawn(Warrior { strength: 42 }).id();

        let s = world.get::<Warrior>(warrior).unwrap();
        assert_eq!(s.strength, 42);

        let tag = world.get::<CombatTag>(warrior);
        assert!(tag.is_some(), "Required CombatTag should be present");
    }

    #[test]
    fn query_finds_all_enemies_in_scene() {
        let mut world = World::new();
        build_village_scene(&mut world);

        let mut query = world.query::<(&Enemy, &Health)>();
        let enemies: Vec<&Health> = query.iter(&world).map(|(_, h)| h).collect();

        assert_eq!(enemies.len(), 1, "Should find exactly 1 enemy");
        assert_eq!(enemies[0].current, 30);
    }
}
