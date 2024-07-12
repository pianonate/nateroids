use bevy::prelude::*;

use crate::{
    asteroids::Asteroid,
    health::Health,
    schedule::InGameSet,
    spaceship::SpaceshipMissile,
};

const DESPAWN_DISTANCE: f32 = 100.0;

pub struct DespawnPlugin;

impl Plugin for DespawnPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (despawn_far_away_entities,despawn_dead_entities).in_set(InGameSet::DespawnEntities),
        );
    }
}

fn despawn_far_away_entities(
    mut commands: Commands,
    query: Query<(Entity, &GlobalTransform), Or<(With<Asteroid>, With<SpaceshipMissile>)>>,
) {
   // println!("entities: {:?}", query.iter().len());
    for (entity, transform) in query.iter() {
        // how far away is the entity from the origin
        let distance = transform.translation().distance(Vec3::ZERO);
        if distance > DESPAWN_DISTANCE {
            println!("distance {:?}", entity);
            despawn(&mut commands, entity);
        }
    }
}

fn despawn(commands: &mut Commands, entity: Entity) {
    commands.entity(entity).despawn_recursive();
}

fn despawn_dead_entities(mut commands: Commands, query: Query<(Entity, &Health)>) {
    for (entity, health) in query.iter() {
        if health.value <= 0.0 {
            despawn(&mut commands, entity);
        }
    }
}

