use crate::asteroids::Asteroid;
use crate::schedule::InGameSet;
use crate::spaceship::SpaceshipMissile;
use bevy::prelude::*;

const DESPAWN_DISTANCE: f32 = 100.0;

pub struct DespawnPlugin;

impl Plugin for DespawnPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            despawn_far_away_entities.in_set(InGameSet::DespawnEntities),
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
            commands.entity(entity).despawn_recursive();
        }
    }
}
