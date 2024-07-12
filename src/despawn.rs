use bevy::prelude::*;

use crate::state::GameState;
use crate::{
    asteroids::Asteroid, health::Health, schedule::InGameSet, spaceship::Spaceship,
    spaceship::SpaceshipMissile,
};

const DESPAWN_DISTANCE: f32 = 100.0;

pub struct DespawnPlugin;

impl Plugin for DespawnPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (despawn_far_away_entities, despawn_dead_entities).in_set(InGameSet::DespawnEntities),
        )
        .add_systems(OnEnter(GameState::GameOver), despawn_all_entities);
    }
}

fn despawn_far_away_entities(
    mut commands: Commands,
    query: Query<
        (Entity, &GlobalTransform),
        Or<(With<Asteroid>, With<SpaceshipMissile>, With<Spaceship>)>,
    >,
) {
    // println!("entities: {:?}", query.iter().len());
    for (entity, transform) in query.iter() {
        // how far away is the entity from the origin
        let distance = transform.translation().distance(Vec3::ZERO);
        if distance > DESPAWN_DISTANCE {
            println!("dead from distance");
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
            println!("dead from collision");
            despawn(&mut commands, entity);
        }
    }
}

fn despawn_all_entities(mut commands: Commands, query: Query<Entity, With<Health>>) {
    for entity in query.iter() {
        println!("dead from GameOver");
        despawn(&mut commands, entity);
    }
}
