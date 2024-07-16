use bevy::{prelude::*, utils::HashMap};
use bevy_rapier3d::prelude::CollisionEvent;

use crate::{
    health::Health,
    schedule::InGameSet,
    spaceship::{Spaceship, SpaceshipMissile},
};

#[derive(Component, Debug)]
pub struct OldCollider {
    pub radius: f32,
    pub colliding_entities: Vec<Entity>,
}

impl OldCollider {
    pub fn new(radius: f32) -> Self {
        Self {
            radius,
            colliding_entities: vec![],
        }
    }
}

#[derive(Component, Debug)]
pub struct CollisionDamage {
    pub amount: f32,
}

impl CollisionDamage {
    pub fn new(amount: f32) -> Self {
        Self { amount }
    }
}

#[derive(Event, Debug)]
pub struct OldCollisionEvent {
    pub entity: Entity,
    pub collided_entity: Entity,
}

impl OldCollisionEvent {
    pub fn new(entity: Entity, collided_entity: Entity) -> Self {
        Self {
            entity,
            collided_entity,
        }
    }
}

pub struct CollisionDetectionPlugin;

impl Plugin for CollisionDetectionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (collision_detection, rapier_collision_damage).in_set(InGameSet::CollisionDetection),
        )
        .add_systems(
            Update,
            (
                (
                    handle_collisions::<Spaceship>,
                    // because asteroids now take damage, we want
                    // missiles to be destroyed on contact so they
                    // don't continue dealing damage every frame to an asteroid
                    handle_collisions::<SpaceshipMissile>,
                ),
                apply_collision_damage,
            )
                .chain()
                .in_set(InGameSet::DespawnEntities),
        )
        .add_event::<OldCollisionEvent>();
    }
}

fn collision_detection(mut query: Query<(Entity, &GlobalTransform, &mut OldCollider)>) {
    let mut colliding_entities: HashMap<Entity, Vec<Entity>> = HashMap::new();

    // First phase: Detect collisions.
    for (entity_a, transform_a, collider_a) in query.iter() {
        for (entity_b, transform_b, collider_b) in query.iter() {
            if entity_a != entity_b {
                let distance = transform_a
                    .translation()
                    .distance(transform_b.translation());
                if distance < collider_a.radius + collider_b.radius {
                    colliding_entities
                        .entry(entity_a)
                        .or_insert_with(Vec::new)
                        .push(entity_b);
                }
            }
        }
    }

    // Second phase: Update colliders.
    for (entity, _, mut collider) in query.iter_mut() {
        collider.colliding_entities.clear();
        if let Some(collisions) = colliding_entities.get(&entity) {
            collider
                .colliding_entities
                .extend(collisions.iter().copied());
        }
    }
}

// I notice that sometimes it can't find the item in the world to do the despawn
// i am pretty sure this happens when the ship runs into an asteroid and also a missile
// hits the asteroid so it has two collisions (or more given timing) so
// that it has queued up multiple collisions despawn
// it's a no-op if it tries so don't worry about it
fn handle_collisions<T: Component>(
    mut collision_event_writer: EventWriter<OldCollisionEvent>,
    query: Query<(Entity, &OldCollider), With<T>>,
    spaceship_query: Query<(), With<Spaceship>>,
    missile_query: Query<(), With<SpaceshipMissile>>,
) {
    for (entity, collider) in query.iter() {
        for &collided_entity in collider.colliding_entities.iter() {
            // Entity collided with another entity of the same type.
            // i.e., query.get essentially asks for the components that
            // the current query returns - in this case if it matches the same T
            // that means we've collided with something of the same type as the handle_collisions
            // query is specifying - so we can ignore it
            if query.get(collided_entity).is_ok() {
                continue;
            }

            // Specific immunity check: skip if a spaceship collides with a missile
            // remember .get does a type check against the query so if we have either the situation
            // of a spaceship having a collision with a missile, or a missile having a collision with a spaceship
            // we can safely ignore that
            if (spaceship_query.get(entity).is_ok() && missile_query.get(collided_entity).is_ok())
                || (missile_query.get(entity).is_ok()
                    && spaceship_query.get(collided_entity).is_ok())
            {
                continue;
            }

            // send a collision event
            collision_event_writer.send(OldCollisionEvent::new(entity, collided_entity));
        }
    }
}

pub fn apply_collision_damage(
    mut collision_event_reader: EventReader<OldCollisionEvent>,
    mut health_query: Query<&mut Health>,
    collision_damage_query: Query<&CollisionDamage>,
) {
    for &OldCollisionEvent {
        entity,
        collided_entity,
    } in collision_event_reader.read()
    {
        let Ok(mut health) = health_query.get_mut(entity) else {
            continue;
        };

        let Ok(collision_damage) = collision_damage_query.get(collided_entity) else {
            continue;
        };

        health.value -= collision_damage.amount;
    }
}

fn rapier_collision_damage(
    mut collision_events: EventReader<CollisionEvent>,
    mut health_query: Query<&mut Health>,
    name_query: Query<&Name>,
    collision_damage_query: Query<&CollisionDamage>,
) {
    for &collision_event in collision_events.read() {
        match collision_event {
            CollisionEvent::Started(entity1, entity2, ..) => {
                // #rust-question: is this the most idiomatic way to do this in rust?
                if apply_rapier_damage(&mut health_query, &collision_damage_query, entity1) {
                    continue;
                }
                if apply_rapier_damage(&mut health_query, &collision_damage_query, entity2) {
                    continue;
                }
                if let Ok(name) = name_query.get(entity1) {
                    println!("{:?}", name);
                }
                if let Ok(name) = name_query.get(entity2) {
                    println!("{:?}", name);
                }
            }
            _ => (),
        }
    }
}

fn apply_rapier_damage(
    health_query: &mut Query<&mut Health>,
    collision_damage_query: &Query<&CollisionDamage>,
    entity: Entity,
) -> bool {
    let Ok(mut health) = health_query.get_mut(entity) else {
        return true;
    };

    let Ok(collision_damage) = collision_damage_query.get(entity) else {
        return true;
    };

    health.value -= collision_damage.amount;

    println!(
        "applied damage {:?} remaining health: {:?}",
        collision_damage.amount, health.value
    );
    false
}
