use crate::camera::PrimaryCamera;
use crate::schedule::InGameSet;

use bevy::prelude::*;
use bevy::window::{PrimaryWindow, WindowResized};

pub struct WindowPlugin;

impl Plugin for WindowPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(First, window_resize_system.in_set(InGameSet::EntityUpdates))
            .add_systems(Startup, initialize_viewport_dimensions);
    }
}

#[derive(Resource, Clone, Copy, Debug)]
pub struct ViewportWorldDimensions {
    pub width: f32,
    pub height: f32,
}

// todo: #bevyquestion - without initializing the ViewportWorldDimensions, the spaceship
//                       spawns at the origin. Is there a better way to handle this?
//                       and to add to this, ViewportWorldDirections is supposed to be updated
//                       by the camera with the camera distance and fov, so these values
//                       seemingly can't be right...but i think i was confused when i first created
//                       this - not realizing the distinction between Viewport and World dimensions
//                       maybe to get this off the ground, it's just the ratio that matters?
fn initialize_viewport_dimensions(
    mut commands: Commands,
    windows: Query<&Window, With<PrimaryWindow>>,
) {
    if let Ok(window) = windows.get_single() {
        commands.insert_resource(ViewportWorldDimensions {
            width: window.width(),
            height: window.height(),
        });
    }
}

// todo: #bevyquestion - executes on resize and shares logic with inspector which also calls
//                       update_world_viewport_dimensions - this seems inelegant - would be nice
//                       to just have one way this works...
fn window_resize_system(
    mut commands: Commands,
    mut resize_events: EventReader<WindowResized>,
    camera_query: Query<(&Projection, &GlobalTransform), With<PrimaryCamera>>,
) {
    for event in resize_events.read() {
        if let Ok((Projection::Perspective(perspective_projection), global_transform)) =
            camera_query.get_single()
        {
            // at this point we know we'll have values
            let viewport_data = ViewportData {
                fov: perspective_projection.fov,
                camera_distance: global_transform.translation().z,
                height: event.height,
                width: event.width,
            };

            update_world_viewport_dimensions(&mut commands, viewport_data);
        }
    }
}

// created this because window resize uses window.height and width
// but when the inspector is shown, the viewport is a subset of the overall window
// not taken up by the inspector docks
// todo: #bevyquestion, #rustquestion - is there a more elegant way to do this?
pub struct ViewportData {
    pub camera_distance: f32,
    pub fov: f32,
    pub height: f32,
    pub width: f32,
}

// ViewportDimensions is used to determine whether we need to teleport
// needs to be kept up to ate on screen resizes
pub fn update_world_viewport_dimensions(commands: &mut Commands, viewport_data: ViewportData) {
    let screen_width = viewport_data.width;
    let screen_height = viewport_data.height;
    // Calculate the aspect ratio
    let aspect_ratio = screen_width / screen_height;

    // the ratio of half of the field of view allows you to use trigonometry (with a right triangle)
    // to get the ratio (with the tan()) of the half height of the fov to the distance from the camera
    // then multiplying by that distance gives you the visible height. multiplying by the aspect_ratio
    // then would give you the width
    let viewable_height = 2.0 * (viewport_data.fov / 2.0).tan() * viewport_data.camera_distance;
    let viewable_width = viewable_height * aspect_ratio;

    let viewport = ViewportWorldDimensions {
        width: viewable_width,
        height: viewable_height,
    };

    commands.insert_resource(viewport);
}
