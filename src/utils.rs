use crate::camera::PrimaryCamera;
use bevy::prelude::*;
use bevy_rapier3d::prelude::Group;

pub const GROUP_SPACESHIP: Group = Group::GROUP_1;
pub const GROUP_ASTEROID: Group = Group::GROUP_2;
pub const GROUP_MISSILE: Group = Group::GROUP_3;

pub fn name_entity(commands: &mut Commands, entity: Entity, name: &str) {
    commands
        .entity(entity)
        .insert(Name::new(format!("{} {}", name, entity)));
}

pub struct ViewableDimensions {
    pub width: f32,
    pub height: f32,
}

pub fn calculate_viewable_dimensions(
    windows: &Query<&Window>,
    camera_query: &Query<(&Projection, &GlobalTransform), With<PrimaryCamera>>,
) -> Option<ViewableDimensions> {
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

                return Some(ViewableDimensions {
                    width: viewable_width,
                    height: viewable_height,
                });
            }
        }
    }
    None
}
