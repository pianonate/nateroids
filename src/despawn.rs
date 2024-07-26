use bevy::prelude::*;

use crate::{health::Health, missile::MissileMovement, schedule::InGameSet, state::GameState};

pub struct DespawnPlugin;

impl Plugin for DespawnPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (despawn_dead_entities, despawn_missiles).in_set(InGameSet::DespawnEntities),
        )
        .add_systems(OnEnter(GameState::GameOver), despawn_all_entities);
    }
}

fn despawn_missiles(mut commands: Commands, query: Query<(Entity, &MissileMovement)>) {
    for (entity, &limited_distance_mover) in query.iter() {
        // Despawn the entity if it has traveled the total distance
        if limited_distance_mover.traveled_distance >= limited_distance_mover.total_distance {
            despawn(&mut commands, entity);
        }
    }
}

fn despawn(commands: &mut Commands, entity: Entity) {
    commands.entity(entity).despawn_recursive();
}

fn despawn_dead_entities(mut commands: Commands, query: Query<(Entity, &Health, &Name)>) {
    for (entity, health, _name) in query.iter() {
        if health.0 <= 0.0 {
            // if !name.contains("Missile") {
            //     println!("{:?} died from poor health: {:?}\n", _name, health);
            // }
            despawn(&mut commands, entity);
        }
    }
}

fn despawn_all_entities(mut commands: Commands, query: Query<Entity, With<Health>>) {
    println!("GameOver");
    for entity in query.iter() {
        despawn(&mut commands, entity);
    }
}
