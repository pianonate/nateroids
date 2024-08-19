use crate::{
    actor::{
        Aabb,
        Teleporter,
    },
    boundary::{
        boundary_config::BoundaryConfig,
        Boundary,
    },
    state::PlayingGame,
};
use bevy::{
    app::{
        App,
        Plugin,
    },
    color::{
        palettes::tailwind,
        Color,
    },
    math::{
        Dir3,
        Vec3,
    },
    prelude::{
        Component,
        Gizmos,
        Query,
        Res,
        Time,
        Transform,
        *,
    },
};
use bevy_rapier3d::dynamics::Velocity;

pub struct WallApproachPlugin;

impl Plugin for WallApproachPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                wall_approach_system,
                draw_approaching_circles,
                draw_emerging_circles,
            )
                .run_if(
                    in_state(PlayingGame), /* .or_else(in_state(GameState::Paused)) */
                ),
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
    pub approach_distance: f32,
    pub distance_to_wall:  f32,
    pub normal:            Dir3,
    pub position:          Vec3,
    pub radius:            f32,
    pub shrink_distance:   f32,
}

fn wall_approach_system(
    mut query: Query<(&Aabb, &Transform, &Velocity, &Teleporter, &mut WallApproachVisual)>,
    boundary: Res<Boundary>,
    time: Res<Time>,
    boundary_config: Res<BoundaryConfig>,
) {
    let boundary_size = boundary.transform.scale.x.min(boundary.transform.scale.y);
    let approach_distance = boundary_size * boundary_config.boundary_distance_approach;
    let shrink_distance = boundary_size * boundary_config.boundary_distance_shrink;

    let delta_time = time.delta_seconds();

    for (aabb, transform, velocity, teleporter, mut visual) in query.iter_mut() {
        // the max dimension of the aabb is actually the diameter - using it as the
        // radius has the circles start out twice as big and then shrink to fit
        // the size of the object minimum size for small objects is preserved
        let radius = aabb.max_dimension().max(boundary_config.smallest_teleport_circle);

        let position = transform.translation;
        let direction = velocity.linvel.normalize_or_zero();

        if let Some(collision_point) = boundary.find_edge_point(position, direction) {
            let distance_to_wall = position.distance(collision_point);
            let normal = boundary.get_normal_for_position(collision_point);

            if distance_to_wall <= approach_distance {
                visual.approaching = Some(BoundaryWall {
                    approach_distance,
                    distance_to_wall,
                    normal,
                    radius,
                    position: collision_point,
                    shrink_distance,
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
                    approach_distance,
                    distance_to_wall: 0.0,
                    normal,
                    position,
                    radius,
                    shrink_distance,
                });
            }
        } else if let Some(ref mut emerging) = visual.emerging {
            emerging.distance_to_wall += velocity.linvel.length() * delta_time;
            if emerging.distance_to_wall > approach_distance {
                visual.emerging = None;
            }
        }
    }
}

fn draw_approaching_circles(q_wall: Query<&WallApproachVisual>, mut gizmos: Gizmos) {
    for visual in q_wall.iter() {
        if let Some(ref approaching) = visual.approaching {
            let max_radius = approaching.radius;
            let min_radius = max_radius * 0.5;

            let radius = if approaching.distance_to_wall > approaching.shrink_distance {
                max_radius
            } else {
                let scale_factor =
                    (approaching.distance_to_wall / approaching.shrink_distance).clamp(0.0, 1.0);
                min_radius + (max_radius - min_radius) * scale_factor
            };

            gizmos.circle(
                approaching.position,
                approaching.normal,
                radius,
                Color::from(tailwind::BLUE_600),
            );
        }
    }
}

fn draw_emerging_circles(q_wall: Query<&WallApproachVisual>, mut gizmos: Gizmos) {
    for visual in q_wall.iter() {
        if let Some(ref emerging) = visual.emerging {
            let radius = if emerging.distance_to_wall <= emerging.shrink_distance {
                emerging.radius //appearance_config.missile_circle_radius
            } else if emerging.distance_to_wall >= emerging.approach_distance {
                0.0 // This will effectively make the circle disappear
            } else {
                // Linear interpolation between full size and zero,
                // but only after exceeding the shrink distance
                let t = (emerging.distance_to_wall - emerging.shrink_distance)
                    / (emerging.approach_distance - emerging.shrink_distance);
                //appearance_config.missile_circle_radius * (1.0 - t)
                emerging.radius * (1.0 - t)
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
