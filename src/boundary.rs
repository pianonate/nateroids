use crate::config::BoundaryGizmos;
use crate::config::GameConfig;
use crate::state::GameState;
use bevy::{color::palettes::css::GREEN, prelude::*};
use bevy_inspector_egui::InspectorOptions;
use std::cell::Cell;

const DEFAULT_CELL_COUNT: UVec3 = UVec3::new(2, 1, 1);
const DEFAULT_CELL_COLOR: Srgba = GREEN;

pub struct BoundaryPlugin;

impl Plugin for BoundaryPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Boundary>()
            .add_systems(Update, draw_boundary.run_if(in_state(GameState::InGame)));
    }
}

fn draw_boundary(
    mut boundary: ResMut<Boundary>,
    config: Res<GameConfig>,
    mut gizmos: Gizmos<BoundaryGizmos>,
) {
    // updating the transform from config so it can be located in one place
    // and also so that it can be dynamically changed with the inspector while the game is running
    // the boundary transform is used both for position but also so the fixed camera
    // can be positioned based on the boundary scale
    boundary.transform.scale = config.boundary_cell_scalar * boundary.cell_count.as_vec3();

    // update the longest diagonal so that the camera can be positioned correctly

    gizmos
        .grid_3d(
            boundary.transform.translation,
            Quat::IDENTITY,
            boundary.cell_count,
            Vec3::splat(config.boundary_cell_scalar),
            DEFAULT_CELL_COLOR,
        )
        .outer_edges();
}

#[derive(Reflect, Resource, Debug, InspectorOptions)]
#[reflect(Resource)]
pub struct Boundary {
    pub cell_count: UVec3,
    pub longest_diagonal: f32,
    pub max_missile_distance: f32,
    pub transform: Transform,
}

impl Default for Boundary {
    fn default() -> Self {
        let cell_scale = GameConfig::default().boundary_cell_scalar * DEFAULT_CELL_COUNT.as_vec3();
        let longest_diagonal =
            (cell_scale.x.powi(2) + cell_scale.y.powi(2) + cell_scale.z.powi(2)).sqrt();

        let max_missile_distance = cell_scale.x.max(cell_scale.y).max(cell_scale.z);

        Self {
            cell_count: DEFAULT_CELL_COUNT,
            longest_diagonal,
            max_missile_distance,
            transform: Transform {
                translation: Vec3::ZERO,
                scale: cell_scale,
                ..Default::default()
            },
        }
    }
}

//impl Boundary {
/// Finds the intersection point of a ray (defined by an origin and direction) with the edges of a viewable area.
///
/// # Parameters
/// - `origin`: The starting point of the ray.
/// - `direction`: The direction vector of the ray.
/// - `dimensions`: The dimensions of the viewable area.
///
/// # Returns
/// An `Option<Vec3>` containing the intersection point if found, or `None` if no valid intersection exists.
///
/// # Method
/// - The function calculates the intersection points of the ray with the positive and negative boundaries of the viewable area along both the x and z axes.
/// - It iterates over these axes, updating the minimum intersection distance (`t_min`) if a valid intersection is found.
/// - Finally, it returns the intersection point corresponding to the minimum distance, or `None` if no valid intersection is found.
impl Boundary {
    pub(crate) fn find_edge_point(&self, origin: Vec3, velocity: Vec3) -> Option<Vec3> {
        let boundary_min = self.transform.translation - self.transform.scale / 2.0;
        let boundary_max = self.transform.translation + self.transform.scale / 2.0;

        // Cell is a type in Rust's standard library that provides interior mutability. It allows
        // you to mutate data even when you have an immutable reference to the Cell. This is useful
        // in scenarios where you need to update a value but only have an immutable reference to the
        // containing structure. In this case it allows us to write a simpler closure
        // that doesn't get littered with & and * - at the cost of using .get() and .set()
        let t_min = Cell::new(f32::MAX);

        for (start, dir, pos_bound, neg_bound) in [
            (origin.x, velocity.x, boundary_max.x, boundary_min.x),
            (origin.y, velocity.y, boundary_max.y, boundary_min.y),
            (origin.z, velocity.z, boundary_max.z, boundary_min.z),
        ] {
            if dir != 0.0 {
                let update_t_min = |boundary: f32| {
                    let t = (boundary - start) / dir;
                    let point = origin + velocity * t;
                    if t > 0.0
                        && t < t_min.get()
                        && is_in_bounds(point, start, origin, boundary_min, boundary_max)
                    {
                        t_min.set(t);
                    }
                };

                update_t_min(pos_bound);
                update_t_min(neg_bound);
            }
        }

        if t_min.get() != f32::MAX {
            let edge_point = origin + velocity * t_min.get();
            return Some(edge_point);
        }
        None
    }
}

fn is_in_bounds(
    point: Vec3,
    start: f32,
    origin: Vec3,
    boundary_min: Vec3,
    boundary_max: Vec3,
) -> bool {
    if start == origin.x {
        point.y >= boundary_min.y
            && point.y <= boundary_max.y
            && point.z >= boundary_min.z
            && point.z <= boundary_max.z
    } else if start == origin.y {
        point.x >= boundary_min.x
            && point.x <= boundary_max.x
            && point.z >= boundary_min.z
            && point.z <= boundary_max.z
    } else {
        point.x >= boundary_min.x
            && point.x <= boundary_max.x
            && point.y >= boundary_min.y
            && point.y <= boundary_max.y
    }
}
