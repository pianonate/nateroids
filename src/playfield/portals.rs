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

pub struct PortalPlugin;

impl Plugin for PortalPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (portal_system, draw_approaching_portals, draw_emerging_portals).run_if(in_state(PlayingGame)),
        );
    }
}

#[derive(Component, Default)]
pub struct ActorPortals {
    pub approaching: Option<Portal>,
    pub emerging:    Option<Portal>,
}

#[derive(Clone, Debug)]
pub struct Portal {
    pub actor_direction:            Vec3,
    pub actor_distance_to_wall:     f32,
    pub boundary_distance_approach: f32,
    pub boundary_distance_shrink:   f32,
    pub boundary_wall_normal:       Dir3,
    pub position:             Vec3,
    pub radius:                     f32,
}

impl Default for Portal {
    fn default() -> Self {
        Self {
            actor_direction:            Vec3::ZERO,
            actor_distance_to_wall:     0.,
            position:             Vec3::ZERO,
            boundary_distance_approach: 0.,
            boundary_distance_shrink:   0.,
            boundary_wall_normal:       Dir3::X,
            radius:                     0.,
        }
    }
}

fn portal_system(
    mut q_actor: Query<(&Aabb, &Transform, &Velocity, &Teleporter, &mut ActorPortals)>,
    boundary: Res<Boundary>,
    boundary_config: Res<Boundary>,
) {
    // todo #handle3d
    let boundary_size = boundary
        .transform
        .scale
        .x
        .min(boundary.transform.scale.y)
        .min(boundary.transform.scale.z);
    let boundary_distance_approach = boundary_size * boundary_config.distance_approach;
    let boundary_distance_shrink = boundary_size * boundary_config.distance_shrink;

    for (aabb, transform, velocity, teleporter, mut visual) in q_actor.iter_mut() {
        // the max dimension of the aabb is actually the diameter - using it as the
        // radius has the circles start out twice as big and then shrink to fit
        // the size of the object minimum size for small objects is preserved
        let radius = aabb.max_dimension().max(boundary_config.portal_smallest);

        let portal_position = transform.translation;
        let actor_direction = velocity.linvel.normalize_or_zero();

        let boundary_wall = Portal {
            actor_direction,
            position: portal_position,
            boundary_distance_approach,
            boundary_distance_shrink,
            radius,
            ..default()
        };

        handle_approaching_visual(&boundary, boundary_wall.clone(), &mut visual);
        handle_emerging_visual(&boundary, boundary_wall.clone(), teleporter, &mut visual);
    }
}

fn handle_emerging_visual(
    boundary: &Res<Boundary>,
    portal: Portal,
    teleporter: &Teleporter,
    visual: &mut Mut<ActorPortals>,
) {
    let approach_distance = portal.boundary_distance_approach;

    if teleporter.just_teleported {
        if let Some(normal) = teleporter.last_teleported_normal {
            // establish the existence of an emerging
            visual.emerging = Some(Portal {
                actor_distance_to_wall: 0.0,
                boundary_wall_normal: normal,
                ..portal
            });
        }
    } else if let Some(ref mut emerging) = visual.emerging {
        let direction = -portal.actor_direction;
        let position = portal.position;

        if let Some(emerging_point) = boundary.find_edge_point(position, direction) {
            // if we established the existence of an emerging point, then we calculate its
            // distance to the wall that is opposite the direction it's
            // traveling from
            emerging.actor_distance_to_wall = position.distance(emerging_point);
            if emerging.actor_distance_to_wall > approach_distance {
                visual.emerging = None;
            }
        }
    }
}

fn handle_approaching_visual(boundary: &Res<Boundary>, portal: Portal, visual: &mut Mut<ActorPortals>) {
    if let Some(collision_point) = boundary.find_edge_point(portal.position, portal.actor_direction) {
        let actor_distance_to_wall = portal.position.distance(collision_point);
        let boundary_wall_normal = boundary.get_normal_for_position(collision_point);

        if actor_distance_to_wall <= portal.boundary_distance_approach {
            let portal_position =
                smooth_circle_position(boundary, visual, collision_point, boundary_wall_normal);

            visual.approaching = Some(Portal {
                actor_distance_to_wall,
                boundary_wall_normal,
                position: portal_position,
                ..portal
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
    visual: &mut Mut<ActorPortals>,
    collision_point: Vec3,
    current_boundary_wall_normal: Dir3,
) -> Vec3 {
    if let Some(approaching) = &visual.approaching {
        // Adjust this value to control smoothing (0.0 to 1.0)
        let smoothing_factor = boundary.portal_movement_smoothing_factor;

        // Only smooth the position if the normal hasn't changed significantly
        // circle_direction_change_factor = threshold for considering normals "similar"
        // approaching carries the last normal, current carries this frame's normal
        if approaching
            .boundary_wall_normal
            .dot(current_boundary_wall_normal.as_vec3())
            > boundary.portal_direction_change_factor
        {
            approaching.position.lerp(collision_point, smoothing_factor)
        } else {
            // If normal changed significantly, jump to new position
            collision_point
        }
    } else {
        collision_point
    }
}

fn draw_approaching_portals(boundary: Res<Boundary>, q_portals: Query<&ActorPortals>, mut gizmos: Gizmos) {
    for portal in q_portals.iter() {
        if let Some(ref approaching) = portal.approaching {
            let max_radius = approaching.radius;
            let min_radius = max_radius * 0.5;

            let radius = if approaching.actor_distance_to_wall > approaching.boundary_distance_shrink {
                max_radius
            } else {
                let scale_factor = (approaching.actor_distance_to_wall
                    / approaching.boundary_distance_shrink)
                    .clamp(0.0, 1.0);
                min_radius + (max_radius - min_radius) * scale_factor
            };

            // draw_portal(&mut gizmos, approaching, radius, Color::from(tailwind::BLUE_600));
            boundary.draw_portal(&mut gizmos, approaching, radius, Color::from(tailwind::BLUE_600))

        }
    }
}

fn draw_emerging_portals(boundary: Res<Boundary>, q_portals: Query<&ActorPortals>, mut gizmos: Gizmos) {
    for portal in q_portals.iter() {
        if let Some(ref emerging) = portal.emerging {
            let radius = if emerging.actor_distance_to_wall <= emerging.boundary_distance_shrink {
                emerging.radius 
            } else if emerging.actor_distance_to_wall >= emerging.boundary_distance_approach {
                0.0 // This will effectively make the circle disappear
            } else {
                // Linear interpolation between full size and zero,
                // but only after exceeding the shrink distance
                let t = (emerging.actor_distance_to_wall - emerging.boundary_distance_shrink)
                    / (emerging.boundary_distance_approach - emerging.boundary_distance_shrink);
                emerging.radius * (1.0 - t)
            };

            if radius > 0.0 {
                // draw_portal(&mut gizmos, emerging, radius, Color::from(tailwind::YELLOW_800));
                boundary.draw_portal(&mut gizmos, emerging, radius, Color::from(tailwind::YELLOW_800))
            }
        }
    }
}

fn draw_portal(gizmos: &mut Gizmos, portal: &Portal, radius: f32, color: Color) {
    gizmos.circle(
        portal.position,
        portal.boundary_wall_normal,
        radius,
        color,
    );
}
