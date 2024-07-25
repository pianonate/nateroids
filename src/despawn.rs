use bevy::prelude::*;

use crate::{
    health::Health,
    movement::{LimitedDistanceMover, Wrappable},
    schedule::InGameSet,
    state::GameState,
};

pub struct DespawnPlugin;

impl Plugin for DespawnPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (despawn_dead_entities, despawn_limited_distance_movers)
                .in_set(InGameSet::DespawnEntities),
        )
        .add_systems(OnEnter(GameState::GameOver), despawn_all_entities);
    }
}

fn despawn_limited_distance_movers(
    mut commands: Commands,
    mut query: Query<(Entity, &mut LimitedDistanceMover, &Wrappable)>,
) {
    for (entity, mut limited_distance_mover, wrappable) in query.iter_mut() {
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
    for (entity, health, name) in query.iter() {
        if health.value <= 0.0 {
            if !name.contains("Missile") {
                println!("{:?} died from poor health: {:?}\n", name, health);
            }
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
