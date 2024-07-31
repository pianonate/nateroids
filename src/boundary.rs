use bevy::{color::palettes::css::GREEN, prelude::*};
use bevy_inspector_egui::InspectorOptions;

const DEFAULT_CELL_SCALE: Vec3 = Vec3::new(75., 75., 75.);
const DEFAULT_CELL_COUNT: UVec3 = UVec3::new(2, 1, 1);
const DEFAULT_CELL_COLOR: Srgba = GREEN;

pub struct BoundaryPlugin;

impl Plugin for BoundaryPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Boundary::default())
            .add_systems(Update, draw_boundary);
    }
}

fn draw_boundary(boundary: Res<Boundary>, mut gizmos: Gizmos) {
    gizmos
        .grid_3d(
            boundary.transform.translation,
            Quat::IDENTITY,
            boundary.cell_count,
            boundary.cell_scale,
            DEFAULT_CELL_COLOR,
        )
        .outer_edges();
}

#[derive(Reflect, Resource, InspectorOptions)]
#[reflect(Resource)]
pub struct Boundary {
    pub cell_count: UVec3,
    pub cell_scale: Vec3,
    pub transform: Transform,
}

impl Default for Boundary {
    fn default() -> Self {
        Self {
            cell_count: DEFAULT_CELL_COUNT,
            cell_scale: DEFAULT_CELL_SCALE,
            transform: Transform {
                translation: Vec3::ZERO,
                scale: DEFAULT_CELL_SCALE * DEFAULT_CELL_COUNT.as_vec3(),
                ..Default::default()
            },
        }
    }
}
