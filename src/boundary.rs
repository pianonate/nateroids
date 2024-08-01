use crate::game_scale::GameScale;
use bevy::{color::palettes::css::GREEN, prelude::*};
use bevy_inspector_egui::InspectorOptions;

const DEFAULT_CELL_SCALE: Vec3 = Vec3::new(175., 175., 175.);
const DEFAULT_CELL_COUNT: UVec3 = UVec3::new(2, 1, 1);
const DEFAULT_CELL_COLOR: Srgba = GREEN;

pub struct BoundaryPlugin;

impl Plugin for BoundaryPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Boundary>()
            .add_systems(Update, draw_boundary);
    }
}

fn draw_boundary(mut boundary: ResMut<Boundary>, game_scale: Res<GameScale>, mut gizmos: Gizmos) {
    // updating the transform from game_scale so it can be located in one place
    // and also so that it can be dynamically changed with the inspector while the game is running
    // the boundary transform is used both for position but also so the fixed camera
    // can be positioned based on the boundary scale
    boundary.transform.scale = game_scale.boundary_cell_scalar * boundary.cell_count.as_vec3();

    gizmos
        .grid_3d(
            boundary.transform.translation,
            Quat::IDENTITY,
            boundary.cell_count,
            Vec3::splat(game_scale.boundary_cell_scalar),
            DEFAULT_CELL_COLOR,
        )
        .outer_edges();
}

#[derive(Reflect, Resource, Debug, InspectorOptions)]
#[reflect(Resource)]
pub struct Boundary {
    pub cell_count: UVec3,
    pub transform: Transform,
}

impl Default for Boundary {
    fn default() -> Self {
        let cell_scale = GameScale::default().boundary_cell_scalar;
        Self {
            cell_count: DEFAULT_CELL_COUNT,
            transform: Transform {
                translation: Vec3::ZERO,
                scale: cell_scale * DEFAULT_CELL_COUNT.as_vec3(),
                ..Default::default()
            },
        }
    }
}
