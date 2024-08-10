use crate::{
    config::{
        AppearanceConfig,
        BoundaryGizmos,
    },
    movement::Teleporter,
    state::GameState,
};
use bevy::{
    color::palettes::css::{
        GREEN,
        RED,
    },
    prelude::{
        Color::Srgba,
        *,
    },
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
                draw_wall_approach_circles,
            )
                .run_if(in_state(GameState::InGame).or_else(in_state(GameState::Paused))),
        );
    }
}

// This component can be added to any entity that should trigger the wall
// approach visualization
#[derive(Component, Default)]
pub struct WallApproachVisual {
    approaching: Option<BoundaryWall>,
    emerging:    Option<BoundaryWall>,
}

struct BoundaryWall {
    normal:   Dir3,
    draw_at:  Vec3,
    distance: f32,
    circles:  usize,
}

// Constants for the visualization
const APPROACH_THRESHOLD: f32 = 0.3; // 20% of the boundary size
const SHRINK_THRESHOLD: f32 = 0.07;
const CIRCLE_COUNT: usize = 10;
const APPROACHING_COLOR: Color = Srgba(RED);
const EMERGING_COLOR: Color = Srgba(GREEN);

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
) {
    let delta_time = time.delta_seconds();
    let boundary_size = boundary.transform.scale.x.min(boundary.transform.scale.y);
    let approach_distance = boundary_size * APPROACH_THRESHOLD;
    let shrink_distance = boundary_size * SHRINK_THRESHOLD;

    for (transform, velocity, teleporter, mut visual) in query.iter_mut() {
        let position = transform.translation;
        let direction = velocity.linvel.normalize_or_zero();

        if let Some(collision_point) = boundary.find_edge_point(position, direction) {
            let distance_to_wall = position.distance(collision_point);

            if distance_to_wall <= approach_distance {
                let normal = boundary.get_normal_for_position(collision_point);
                // let active_circles = ((CIRCLE_COUNT as f32)
                //     * (1.0 - distance_to_wall / shrink_distance))
                //     .ceil() as usize;
                let active_circles = if distance_to_wall <= shrink_distance {
                    // Calculate shrinking circles only within shrink distance
                    ((CIRCLE_COUNT as f32) * (1.0 - distance_to_wall / shrink_distance)).ceil()
                        as usize
                } else {
                    // Only one circle between approach and shrink distance
                    1
                };
                visual.approaching = Some(BoundaryWall {
                    normal,
                    draw_at: collision_point,
                    distance: distance_to_wall,
                    circles: active_circles.min(CIRCLE_COUNT),
                });
                visual.emerging = None;
            } else {
                visual.approaching = None;

                if teleporter.just_teleported {
                    if let Some(normal) = teleporter.last_teleported_normal {
                        // boundary.get_normal_for_position(position);
                        visual.emerging = Some(BoundaryWall {
                            normal,
                            draw_at: position,
                            distance: 0.0,
                            circles: CIRCLE_COUNT,
                        });
                    }
                } else if let Some(ref mut emerging) = visual.emerging {
                    emerging.distance += velocity.linvel.length() * delta_time;
                    if emerging.distance > shrink_distance {
                        visual.emerging = None;
                    } else {
                        let progress = emerging.distance / shrink_distance;
                        emerging.circles = (CIRCLE_COUNT as f32 * (1.0 - progress)).ceil() as usize;
                    }
                }
            }
        } else {
            visual.approaching = None;
            visual.emerging = None;
        }
    }
}

pub fn draw_wall_approach_circles(
    query: Query<&WallApproachVisual>,
    boundary: Res<Boundary>,
    mut gizmos: Gizmos,
) {
    let boundary_size = boundary.transform.scale.x.min(boundary.transform.scale.y);
    let threshold_distance = boundary_size * SHRINK_THRESHOLD;

    for visual in query.iter() {
        if let Some(ref approaching) = visual.approaching {
            draw_approaching_circles(
                &mut gizmos,
                approaching.draw_at,
                approaching.normal,
                threshold_distance,
                APPROACHING_COLOR,
                approaching.circles,
            );
        }

        if let Some(ref emerging) = visual.emerging {
            draw_emerging_circles(
                &mut gizmos,
                emerging.draw_at,
                emerging.normal,
                threshold_distance,
                EMERGING_COLOR,
                emerging.circles,
            );
        }
    }
}

fn draw_approaching_circles(
    gizmos: &mut Gizmos,
    position: Vec3,
    normal: Dir3,
    max_radius: f32,
    color: Color,
    circle_count: usize,
) {
    for i in 0..circle_count {
        let radius = max_radius * (CIRCLE_COUNT - i) as f32 / CIRCLE_COUNT as f32;
        let dir3_normal = normal;
        gizmos.circle(position, dir3_normal, radius, color);
    }
}

fn draw_emerging_circles(
    gizmos: &mut Gizmos,
    position: Vec3,
    normal: Dir3,
    max_radius: f32,
    color: Color,
    remaining_circles: usize,
) {
    for i in 0..remaining_circles {
        let radius = max_radius * (i + 1) as f32 / CIRCLE_COUNT as f32;
        let dir3_normal = normal; //Dir3::new(normal).unwrap_or(Dir3::NEG_Z);
        gizmos.circle(position, dir3_normal, radius, color);
    }
}
