use bevy::prelude::*;

use crate::state::GameState;
use crate::{health::Health, schedule::InGameSet};

const OLD_AGE: u32 = 120;

#[derive(Component, Debug)]
pub struct Mortal {
    age: u32,
}

impl Mortal {
    pub fn new(age: u32) -> Self {
        Self { age }
    }
}

pub struct DespawnPlugin;

impl Plugin for DespawnPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (despawn_aged_entities, despawn_dead_entities).in_set(InGameSet::DespawnEntities),
        )
        .add_systems(OnEnter(GameState::GameOver), despawn_all_entities);
    }
}

fn despawn_aged_entities(mut commands: Commands, mut query: Query<(Entity, &mut Mortal)>) {
    for (entity, mut mortal) in query.iter_mut() {
        mortal.age += 1;

        if mortal.age > OLD_AGE {
            // println!("dead from age");
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
            // println!("dead from collision");
            despawn(&mut commands, entity);
        }
    }
}

fn despawn_all_entities(mut commands: Commands, query: Query<Entity, With<Health>>) {
    for entity in query.iter() {
        // println!("dead from GameOver");
        despawn(&mut commands, entity);
    }
}
