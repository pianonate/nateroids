use crate::{
    config::{
        AppearanceConfig,
        BoundaryGizmos,
    },
    movement::Teleporter,
    state::GameState,
};
use bevy::{
    color::palettes::tailwind,
    prelude::*,
};
use bevy_inspector_egui::InspectorOptions;
use bevy_rapier3d::prelude::Velocity;
use std::cell::Cell;

pub struct BoundaryPlugin;

impl Plugin for BoundaryPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Boundary>().add_systems(
            Update,
            (
                draw_boundary,
                wall_approach_system,
                // draw_wall_approach_circles,
                draw_approaching_circles,
                draw_emerging_circles,
            )
                .run_if(in_state(GameState::InGame).or_else(in_state(GameState::Paused))),
        );
    }
}

#[derive(Component, Default)]
pub struct WallApproachVisual {
    pub approaching: Option<BoundaryWall>,
    pub emerging:    Option<BoundaryWall>,
}

#[derive(Clone, Debug)]
pub struct BoundaryWall {
    pub position: Vec3,
    pub normal:   Dir3,
    pub distance: f32,
}

fn draw_boundary(
    mut boundary: ResMut<Boundary>,
    config: Res<AppearanceConfig>,
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

#[derive(Reflect, Resource, Debug, InspectorOptions)]
#[reflect(Resource)]
pub struct Boundary {
    pub cell_count:           UVec3,
    pub longest_diagonal:     f32,
    pub max_missile_distance: f32,
    pub transform:            Transform,
}

impl Default for Boundary {
    fn default() -> Self {
        let config = AppearanceConfig::default();

        let cell_scale = config.boundary_scalar * config.boundary_cell_count.as_vec3();
        let longest_diagonal =
            (cell_scale.x.powi(2) + cell_scale.y.powi(2) + cell_scale.z.powi(2)).sqrt();

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
///   positive and negative boundaries of the viewable area along both the x and
///   z axes.
/// - It iterates over these axes, updating the minimum intersection distance
///   (`t_min`) if a valid intersection is found.
/// - Finally, it returns the intersection point corresponding to the minimum
///   distance, or `None` if no valid intersection is found.
impl Boundary {
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

    pub fn find_edge_point(&self, origin: Vec3, velocity: Vec3) -> Option<Vec3> {
        let boundary_min = self.transform.translation - self.transform.scale / 2.0;
        let boundary_max = self.transform.translation + self.transform.scale / 2.0;

        // Cell is a type in Rust's standard library that provides interior mutability.
        // It allows you to mutate data even when you have an immutable
        // reference to the Cell. This is useful in scenarios where you need to
        // update a value but only have an immutable reference to the containing
        // structure. In this case it allows us to write a simpler closure
        // that doesn't get littered with & and * - at the cost of using .get() and
        // .set()
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

pub fn wall_approach_system(
    mut query: Query<(&Transform, &Velocity, &Teleporter, &mut WallApproachVisual)>,
    boundary: Res<Boundary>,
    time: Res<Time>,
    appearance: Res<AppearanceConfig>,
) {
    let boundary_size = boundary.transform.scale.x.min(boundary.transform.scale.y);
    let approach_distance = boundary_size * appearance.boundary_distance_approach;
    let delta_time = time.delta_seconds();

    for (transform, velocity, teleporter, mut visual) in query.iter_mut() {
        let position = transform.translation;
        let direction = velocity.linvel.normalize_or_zero();

        if let Some(collision_point) = boundary.find_edge_point(position, direction) {
            let distance_to_wall = position.distance(collision_point);
            let normal = boundary.get_normal_for_position(collision_point);

            if distance_to_wall <= approach_distance {
                visual.approaching = Some(BoundaryWall {
                    position: collision_point,
                    normal,
                    distance: distance_to_wall,
                });
                visual.emerging = None;
            } else {
                visual.approaching = None;
            }
        } else {
            visual.approaching = None;
        }

        if teleporter.just_teleported {
            if let Some(normal) = teleporter.last_teleported_normal {
                visual.emerging = Some(BoundaryWall {
                    position,
                    normal,
                    distance: 0.0,
                });
            }
        } else if let Some(ref mut emerging) = visual.emerging {
            emerging.distance += velocity.linvel.length() * delta_time;
            if emerging.distance > approach_distance {
                visual.emerging = None;
            }
        }
    }
}

fn draw_approaching_circles(
    query: Query<&WallApproachVisual>,
    boundary: Res<Boundary>,
    mut gizmos: Gizmos,
    appearance_config: Res<AppearanceConfig>,
) {
    let boundary_size = boundary.transform.scale.x.min(boundary.transform.scale.y);
    let shrink_distance = boundary_size * appearance_config.boundary_distance_shrink;

    for visual in query.iter() {
        if let Some(ref approaching) = visual.approaching {
            draw_single_circle(
                &mut gizmos,
                approaching.position,
                approaching.normal,
                approaching.distance,
                Color::from(tailwind::BLUE_600),
                &appearance_config,
                shrink_distance,
            );
        }
    }
}

fn draw_emerging_circles(
    query: Query<&WallApproachVisual>,
    boundary: Res<Boundary>,
    mut gizmos: Gizmos,
    appearance_config: Res<AppearanceConfig>,
) {
    let boundary_size = boundary.transform.scale.x.min(boundary.transform.scale.y);
    let shrink_distance = boundary_size * appearance_config.boundary_distance_shrink;
    let approach_distance = boundary_size * appearance_config.boundary_distance_approach;

    for visual in query.iter() {
        if let Some(ref emerging) = visual.emerging {
            let radius = if emerging.distance <= shrink_distance {
                appearance_config.missile_circle_radius
            } else if emerging.distance >= approach_distance {
                0.0 // This will effectively make the circle disappear
            } else {
                // Linear interpolation between full size and zero,
                // but only after exceeding the shrink distance
                let t =
                    (emerging.distance - shrink_distance) / (approach_distance - shrink_distance);
                appearance_config.missile_circle_radius * (1.0 - t)
            };

            if radius > 0.0 {
                gizmos.circle(
                    emerging.position,
                    emerging.normal,
                    radius,
                    Color::from(tailwind::YELLOW_800),
                );
            }
        }
    }
}

fn draw_single_circle(
    gizmos: &mut Gizmos,
    position: Vec3,
    normal: Dir3,
    distance: f32,
    color: Color,
    config: &AppearanceConfig,
    shrink_distance: f32,
) {
    let max_radius = config.missile_circle_radius;
    let min_radius = max_radius * 0.5;

    let radius = if distance > shrink_distance {
        max_radius
    } else {
        let scale_factor = (distance / shrink_distance).clamp(0.0, 1.0);
        min_radius + (max_radius - min_radius) * scale_factor
    };

    gizmos.circle(position, normal, radius, color);
}
