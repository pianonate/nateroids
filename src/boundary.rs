use bevy::color::palettes::css::RED;
use bevy::prelude::Color::Srgba;
use bevy::prelude::*;

const DEFAULT_BOUNDARY: f32 = 75.0;

pub struct BoundaryPlugin;

impl Plugin for BoundaryPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Boundary::default())
            .add_systems(Update, draw_boundary);
    }
}

fn draw_boundary(boundary: Res<Boundary>, mut gizmos: Gizmos) {
    gizmos.cuboid(boundary.transform, Srgba(RED));
}

#[derive(Resource)]
pub struct Boundary {
    pub(crate) transform: Transform,
}

impl Default for Boundary {
    fn default() -> Self {
        Self {
            transform: Transform {
                translation: Vec3::ZERO,
                scale: Vec3::splat(DEFAULT_BOUNDARY),
                ..Default::default()
            },
        }
    }
}
