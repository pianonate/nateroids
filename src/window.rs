use crate::camera::PrimaryCamera;
use crate::schedule::InGameSet;
use bevy::prelude::*;
use bevy::window::WindowResized;

pub struct WindowPlugin;

impl Plugin for WindowPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            window_resize_system.in_set(InGameSet::DespawnEntities),
        )
        .add_systems(Startup, initialize_viewport_dimensions);
    }
}

#[derive(Resource, Clone, Copy, Debug)]
pub struct ViewportDimensions {
    pub width: f32,
    pub height: f32,
}

// todo: #bevyquestion - there's only one window but we iterate it anyway - is this idiomatic?
fn initialize_viewport_dimensions(mut commands: Commands, windows: Query<&Window>) {
    for window in windows.iter() {
        commands.insert_resource(ViewportDimensions {
            width: window.width(),
            height: window.height(),
        });
    }
}

fn window_resize_system(
    mut commands: Commands,
    mut resize_events: EventReader<WindowResized>,
    windows: Query<&Window>,
    camera_query: Query<(&Projection, &GlobalTransform), With<PrimaryCamera>>,
) {
    for event in resize_events.read() {
        if let Some(viewport) = calculate_viewport(&windows, &camera_query) {
            println!(
                "Window resized to: {}x{} with viewport: {}x{} ",
                event.width, event.height, viewport.width, viewport.height
            );
            commands.insert_resource(ViewportDimensions {
                width: viewport.width,
                height: viewport.height,
            });
        }
    }
}

/// given a particular camera, what is the viewable/width and height for that camera?
fn calculate_viewport(
    windows: &Query<&Window>,
    camera_query: &Query<(&Projection, &GlobalTransform), With<PrimaryCamera>>,
) -> Option<ViewportDimensions> {
    if let Ok(window) = windows.get_single() {
        let screen_width = window.width();
        let screen_height = window.height();
        // Calculate the aspect ratio
        let aspect_ratio = screen_width / screen_height;

        if let Ok((Projection::Perspective(perspective_projection), global_transform)) =
            camera_query.get_single()
        {
            // Calculate the viewable width and height at the plane level
            let camera_distance = global_transform.translation().y;
            let viewable_height = 2.0 * (perspective_projection.fov / 2.0).tan() * camera_distance;
            let viewable_width = viewable_height * aspect_ratio;

            return Some(ViewportDimensions {
                width: viewable_width,
                height: viewable_height,
            });
        }
    }
    None
}