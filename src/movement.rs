use crate::{camera::PrimaryCamera, collision_detection::OldCollider, schedule::InGameSet};
use bevy::prelude::*;

pub struct MovementPlugin;
impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (update_velocity, update_position, wrap_position) // these should happen in order
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
pub struct Wrappable;

#[derive(Component, Debug)]
pub enum MoverType {
    Asteroid,
    Missile,
    Spaceship,
}

#[derive(Bundle)]
pub struct MovingObjectBundle {
    pub acceleration: Acceleration,
    pub collider: OldCollider,
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

// needs to move from world to screen
// remember this game is currently operating int he x-z plane and the a camera is above it
// on the y axis
fn wrap_position(
    windows: Query<&Window>,
    camera_query: Query<(&Projection, &GlobalTransform), With<PrimaryCamera>>,
    mut wrappable_entities: Query<&mut Transform, With<Wrappable>>,
) {
    if let Ok(window) = windows.get_single() {
        if let Ok((projection, global_transform)) = camera_query.get_single() {
            if let Projection::Perspective(perspective_projection) = projection {
                let screen_width = window.width() as f32;
                let screen_height = window.height() as f32;

                // Calculate the aspect ratio
                let aspect_ratio = screen_width / screen_height;

                // Calculate the viewable width and height at the plane level
                let camera_distance = global_transform.translation().y;
                let viewable_height =
                    2.0 * (perspective_projection.fov / 2.0).tan() * camera_distance;
                let viewable_width = viewable_height * aspect_ratio;

                for mut transform in wrappable_entities.iter_mut() {
                    let x = transform.translation.x;
                    let z = transform.translation.z;

                    let screen_right = viewable_width / 2.0;
                    let screen_left = -screen_right;
                    let screen_top = viewable_height / 2.0;
                    let screen_bottom = -screen_top;

                    if x > screen_right {
                        transform.translation.x = screen_left;
                    } else if x < screen_left {
                        transform.translation.x = screen_right;
                    }

                    if z > screen_top {
                        transform.translation.z = screen_bottom;
                    } else if z < screen_bottom {
                        transform.translation.z = screen_top;
                    }
                }
            } else {
                println!("Projection is not PerspectiveProjection");
            }
        } else {
            println!("Failed to get camera components");
        }
    }
}
