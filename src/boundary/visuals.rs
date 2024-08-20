use crate::{
    boundary::boundary_config::BoundaryConfig,
    camera::RenderLayer,
    // computed states, so not using GameState directly
    state::PlayingGame,
};

use bevy::{
    prelude::*,
    render::view::RenderLayers,
};
use std::cell::Cell;

pub struct VisualsPlugin;

impl Plugin for VisualsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Boundary>()
            .init_gizmo_group::<BoundaryGizmos>()
            .add_systems(Startup, init_gizmo_configs)
            .add_systems(Update, draw_boundary.run_if(in_state(PlayingGame)));
    }
}

#[derive(Default, Reflect, GizmoConfigGroup)]
pub struct BoundaryGizmos {}

fn init_gizmo_configs(mut config_store: ResMut<GizmoConfigStore>, boundary_config: Res<BoundaryConfig>) {
    for (_, any_config, _) in config_store.iter_mut() {
        any_config.render_layers = RenderLayers::from_layers(RenderLayer::Game.layers());
        any_config.line_width = 2.;
    }

    // so we can avoid an error of borrowing the mutable config_store twice
    // in the same context
    {
        let (config, _) = config_store.config_mut::<BoundaryGizmos>();
        config.line_width = boundary_config.boundary_line_width;
    }
}

#[derive(Reflect, Resource, Debug)]
#[reflect(Resource)]
pub struct Boundary {
    pub cell_count:           UVec3,
    pub longest_diagonal:     f32,
    pub max_missile_distance: f32,
    pub transform:            Transform,
}

impl Default for Boundary {
    fn default() -> Self {
        let config = BoundaryConfig::default();

        let cell_scale = config.boundary_scalar * config.boundary_cell_count.as_vec3();
        let longest_diagonal = (cell_scale.x.powi(2) + cell_scale.y.powi(2) + cell_scale.z.powi(2)).sqrt();

        let max_missile_distance = cell_scale.x.max(cell_scale.y).max(cell_scale.z);

        Self {
            cell_count: config.boundary_cell_count,
            longest_diagonal,
            max_missile_distance,
            transform: Transform {
                scale: cell_scale,
                ..Default::default()
            },
        }
    }
}

/// Finds the intersection point of a ray (defined by an origin and direction)
/// with the edges of a viewable area.
///
/// # Parameters
/// - `origin`: The starting point of the ray.
/// - `direction`: The direction vector of the ray.
/// - `dimensions`: The dimensions of the viewable area.
///
/// # Returns
/// An `Option<Vec3>` containing the intersection point if found, or `None` if
/// no valid intersection exists.
///
/// # Method
/// - The function calculates the intersection points of the ray with the
///   positive and negative boundaries of the viewable area along all axes.
///   todo: is this true? you'll have to test in 3d mode
/// - It iterates over these axes, updating the minimum intersection distance
///   (`t_min`) if a valid intersection is found.
/// - Finally, it returns the intersection point corresponding to the minimum
///   distance, or `None` if no valid intersection is found.
impl Boundary {
    pub fn calculate_teleport_position(&self, position: Vec3) -> Vec3 {
        let boundary_min = self.transform.translation - self.transform.scale / 2.0;
        let boundary_max = self.transform.translation + self.transform.scale / 2.0;

        let mut wrapped_position = position;

        if position.x >= boundary_max.x {
            wrapped_position.x = boundary_min.x;
        } else if position.x <= boundary_min.x {
            wrapped_position.x = boundary_max.x;
        }

        if position.y >= boundary_max.y {
            wrapped_position.y = boundary_min.y;
        } else if position.y <= boundary_min.y {
            wrapped_position.y = boundary_max.y;
        }

        if position.z >= boundary_max.z {
            wrapped_position.z = boundary_min.z;
        } else if position.z <= boundary_min.z {
            wrapped_position.z = boundary_max.z;
        }

        wrapped_position
    }

    pub fn get_normal_for_position(&self, position: Vec3) -> Dir3 {
        let half_size = self.transform.scale / 2.0;
        let boundary_min = self.transform.translation - half_size;
        let boundary_max = self.transform.translation + half_size;

        let epsilon = 0.001; // Small value to account for floating-point imprecision

        if (position.x - boundary_min.x).abs() < epsilon {
            Dir3::NEG_X
        } else if (position.x - boundary_max.x).abs() < epsilon {
            Dir3::X
        } else if (position.y - boundary_min.y).abs() < epsilon {
            Dir3::NEG_Y
        } else if (position.y - boundary_max.y).abs() < epsilon {
            Dir3::Y
        } else if (position.z - boundary_min.z).abs() < epsilon {
            Dir3::NEG_Z
        } else if (position.z - boundary_max.z).abs() < epsilon {
            Dir3::Z
        } else {
            // Default to Y if not on a boundary face
            Dir3::Y
        }
    }

    pub fn find_edge_point(&self, origin: Vec3, direction: Vec3) -> Option<Vec3> {
        let boundary_min = self.transform.translation - self.transform.scale / 2.0;
        let boundary_max = self.transform.translation + self.transform.scale / 2.0;

        // Cell is a type in Rust's standard library that provides interior mutability.
        // It allows you toF mutate data even when you have an immutable
        // reference to the Cell. This is useful in scenarios where you need to
        // update a value but only have an immutable reference to the containing
        // structure. In this case it allows us to write a simpler closure
        // that doesn't get littered with & and * - at the cost of using .get() and
        // .set()
        let t_min = Cell::new(f32::MAX);

        for (start, dir, pos_bound, neg_bound) in [
            (origin.x, direction.x, boundary_max.x, boundary_min.x),
            (origin.y, direction.y, boundary_max.y, boundary_min.y),
            (origin.z, direction.z, boundary_max.z, boundary_min.z),
        ] {
            if dir != 0.0 {
                let update_t_min = |boundary: f32| {
                    let t = (boundary - start) / dir;
                    let point = origin + direction * t;
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
            let edge_point = origin + direction * t_min.get();
            return Some(edge_point);
        }
        None
    }
}

fn is_in_bounds(point: Vec3, start: f32, origin: Vec3, boundary_min: Vec3, boundary_max: Vec3) -> bool {
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

fn draw_boundary(
    mut boundary: ResMut<Boundary>,
    config: Res<BoundaryConfig>,
    mut gizmos: Gizmos<BoundaryGizmos>,
) {
    // updating the transform from config so it can be located in one place
    // and also so that it can be dynamically changed with the inspector while the
    // game is running the boundary transform is used both for position but also
    // so the fixed camera can be positioned based on the boundary scale
    boundary.transform.scale = config.boundary_scalar * boundary.cell_count.as_vec3();

    // update the longest diagonal so that the camera can be positioned correctly

    gizmos
        .grid_3d(
            boundary.transform.translation,
            Quat::IDENTITY,
            boundary.cell_count,
            Vec3::splat(config.boundary_scalar),
            config.boundary_color,
        )
        .outer_edges();
}
