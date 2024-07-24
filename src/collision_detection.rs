use bevy::prelude::*;
use bevy_rapier3d::prelude::CollisionEvent;

use crate::{health::Health, schedule::InGameSet};

#[derive(Component, Debug)]
pub struct CollisionDamage {
    pub amount: f32,
}

impl CollisionDamage {
    pub fn new(amount: f32) -> Self {
        Self { amount }
    }
}

pub struct CollisionDetectionPlugin;

impl Plugin for CollisionDetectionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            rapier_collision_damage.in_set(InGameSet::CollisionDetection),
        );
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
                if let Ok(name1) = name_query.get(entity1) {
                    if let Ok(name2) = name_query.get(entity2) {
                        apply_rapier_damage(
                            &mut health_query,
                            &collision_damage_query,
                            entity1,
                            name1,
                            entity2,
                            name2,
                        );
                        apply_rapier_damage(
                            &mut health_query,
                            &collision_damage_query,
                            entity2,
                            name2,
                            entity1,
                            name1,
                        );
                    }
                }
            }
            _ => (),
        }
    }
}

fn apply_rapier_damage(
    health_query: &mut Query<&mut Health>,
    collision_damage_query: &Query<&CollisionDamage>,
    applying_entity: Entity,
    applying_entity_name: &Name,
    receiving_entity: Entity,
    receiving_entity_name: &Name,
) {
    if let Ok(mut health) = health_query.get_mut(receiving_entity) {
        if let Ok(collision_damage) = collision_damage_query.get(applying_entity) {
            health.value -= collision_damage.amount;
            println!(
                "{:?} applied {:?} damage to {:?} leaving it with remaining health: {:?}",
                applying_entity_name, collision_damage.amount, receiving_entity_name, health.value
            );
        }
    }
}
