use crate::{
    actor::{
        Aabb,
        Teleporter,
    },
    playfield::Boundary,
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
    prelude::*,
};
use bevy_rapier3d::dynamics::Velocity;

pub struct WallPortalPlugin;

impl Plugin for WallPortalPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                wall_portal_system,
                draw_approaching_portals,
                draw_emerging_portals,
            )
                .run_if(in_state(PlayingGame)),
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
    pub actor_direction:   Vec3,
    pub approach_distance: f32,
    pub distance_to_wall:  f32,
    pub normal:            Dir3,
    pub position:          Vec3,
    pub radius:            f32,
    pub shrink_distance:   f32,
}

impl Default for BoundaryWall {
    fn default() -> Self {
        Self {
            actor_direction:   Vec3::ZERO,
            approach_distance: 0.,
            distance_to_wall:  0.,
            normal:            Dir3::X,
            position:          Vec3::ZERO,
            radius:            0.,
            shrink_distance:   0.,
        }
    }
}

fn wall_portal_system(
    mut q_actor: Query<(&Aabb, &Transform, &Velocity, &Teleporter, &mut WallApproachVisual)>,
    boundary: Res<Boundary>,
    boundary_config: Res<Boundary>,
) {
    let boundary_size = boundary.transform.scale.x.min(boundary.transform.scale.y);
    let approach_distance = boundary_size * boundary_config.distance_approach;
    let shrink_distance = boundary_size * boundary_config.distance_shrink;

    for (aabb, transform, velocity, teleporter, mut visual) in q_actor.iter_mut() {
        // the max dimension of the aabb is actually the diameter - using it as the
        // radius has the circles start out twice as big and then shrink to fit
        // the size of the object minimum size for small objects is preserved
        let radius = aabb.max_dimension().max(boundary_config.portal_smallest);

        let position = transform.translation;
        let actor_direction = velocity.linvel.normalize_or_zero();

        let boundary_wall = BoundaryWall {
            actor_direction,
            approach_distance,
            radius,
            position,
            shrink_distance,
            ..default()
        };

        handle_approaching_visual( &boundary, boundary_wall.clone(), &mut visual );
        handle_emerging_visual(&boundary, boundary_wall.clone(), teleporter, &mut visual);
    }
}

fn handle_emerging_visual(
    boundary: &Res<Boundary>,
    boundary_wall: BoundaryWall,
    teleporter: &Teleporter,
    visual: &mut Mut<WallApproachVisual>,
) {
    let approach_distance = boundary_wall.approach_distance;
    let position = boundary_wall.position;

    if teleporter.just_teleported {
        if let Some(normal) = teleporter.last_teleported_normal {
            // establish the existence of an emerging
            visual.emerging = Some(BoundaryWall {
                distance_to_wall: 0.0,
                normal,
                ..boundary_wall
            });
        }
    } else if let Some(ref mut emerging) = visual.emerging {

        let direction = -boundary_wall.actor_direction;

        if let Some(emerging_point) = boundary.find_edge_point(position, direction) {
            // if we established the existence of an emerging point, then we calculate its
            // distance to the wall that is opposite the direction it's
            // traveling from
            emerging.distance_to_wall = position.distance(emerging_point);
            if emerging.distance_to_wall > approach_distance {
                visual.emerging = None;
            }
        }
    }
}

fn handle_approaching_visual(
    boundary: &Res<Boundary>,
    boundary_wall: BoundaryWall,
    visual: &mut Mut<WallApproachVisual>,
) {
    if let Some(collision_point) = boundary.find_edge_point(boundary_wall.position, boundary_wall.actor_direction)
    {
        let distance_to_wall = boundary_wall.position.distance(collision_point);
        let normal = boundary.get_normal_for_position(collision_point);

        if distance_to_wall <= boundary_wall.approach_distance {
            let new_position = smooth_circle_position(boundary, visual, collision_point, normal);

            visual.approaching = Some(BoundaryWall {
                distance_to_wall,
                normal,
                position: new_position,
                ..boundary_wall
            });
        } else {
            visual.approaching = None;
        }
    } else {
        visual.approaching = None;
    }
}

// updated to handle two situations
// 1. if you switch direction on approach, the circle used to jump away fast
// implemented a smoothing factor to alleviate this
//
// 2. with the smoothing factor, it can cause the circle to draw on the wrong
//    wall if
// you are close to two walls and switch from the one to the other
// so we need to switch to the new collision point in that case
//
// extracted for readability/complexity
fn smooth_circle_position(
    boundary: &Res<Boundary>,
    visual: &mut Mut<WallApproachVisual>,
    collision_point: Vec3,
    normal: Dir3,
) -> Vec3 {
    if let Some(approaching) = &visual.approaching {

        // Adjust this value to control smoothing (0.0 to 1.0)
        let smoothing_factor = boundary.portal_movement_smoothing_factor;

        // Only smooth the position if the normal hasn't changed significantly
        // circle_direction_change_factor = threshold for considering normals "similar"
        if approaching.normal.dot(normal.as_vec3()) > boundary.portal_direction_change_factor {
            approaching.position.lerp(collision_point, smoothing_factor)
        } else {
            // If normal changed significantly, jump to new position
            collision_point
        }
    } else {
        collision_point
    }
}

fn draw_approaching_portals(q_wall: Query<&WallApproachVisual>, mut gizmos: Gizmos) {
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

            draw_portal(&mut gizmos, approaching, radius, Color::from(tailwind::BLUE_600));

        }
    }
}

fn draw_emerging_portals(q_wall: Query<&WallApproachVisual>, mut gizmos: Gizmos) {
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
                draw_portal(&mut gizmos, emerging, radius, Color::from(tailwind::YELLOW_800));
            }
        }
    }
}

fn draw_portal(gizmos: &mut Gizmos, emerging: &BoundaryWall, radius: f32, color: Color) {
    gizmos.circle(
        emerging.position,
        emerging.normal,
        radius,
        color,
    );
}
