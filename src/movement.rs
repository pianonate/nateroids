use crate::{collision_detection::Collider, schedule::InGameSet};
use bevy::prelude::*;

pub struct MovementPlugin;

impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (update_velocity, update_position) // these should happen in order
                .chain()
                .in_set(InGameSet::EntityUpdates), // use system sets to put this into an enum that controls ordering
        );
    }
}

#[derive(Component, Debug)]
pub struct Velocity {
    pub value: Vec3,
}

impl Velocity {
    pub fn new(value: Vec3) -> Self {
        Self { value }
    }
}

#[derive(Component, Debug)]
pub struct Acceleration {
    pub value: Vec3,
}

impl Acceleration {
    pub fn new(value: Vec3) -> Self {
        Self { value }
    }
}

#[derive(Component, Debug)]
pub enum MoverType {
    Asteroid,
    Missile,
    Spaceship,
}

#[derive(Bundle)]
pub struct MovingObjectBundle {
    pub acceleration: Acceleration,
    pub collider: Collider,
    pub model: SceneBundle,
    pub velocity: Velocity,
    pub mover_type: MoverType,
}

fn update_velocity(mut query: Query<(&Acceleration, &mut Velocity)>, time: Res<Time>) {
    for (acceleration, mut velocity) in query.iter_mut() {
        velocity.value += acceleration.value * time.delta_seconds();
    }
}

fn update_position(mut query: Query<(&Velocity, &mut Transform)>, time: Res<Time>) {
    for (velocity, mut transform) in query.iter_mut() {
        transform.translation += velocity.value * time.delta_seconds();
    }
}
