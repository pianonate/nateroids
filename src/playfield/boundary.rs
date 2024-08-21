use crate::global_input::{
    toggle_active,
    GlobalAction,
};
use crate::{
    camera::RenderLayer,
    // computed states, so not using GameState directly
    state::PlayingGame,
};
use bevy::{
    prelude::*,
    render::view::RenderLayers,
};
use bevy_inspector_egui::{
    inspector_options::std_options::NumberDisplay,
    prelude::*,
    quick::ResourceInspectorPlugin,
};

use bevy::color::palettes::tailwind;

pub struct BoundaryPlugin;

impl Plugin for BoundaryPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Boundary>()
            .init_gizmo_group::<BoundaryGizmos>()
            .register_type::<Boundary>()
            .add_plugins(
                ResourceInspectorPlugin::<Boundary>::default()
                    .run_if(toggle_active(false, GlobalAction::BoundaryInspector)),
            )
            .add_systems(Update, update_gizmos_config)
            .add_systems(Update, draw_boundary.run_if(in_state(PlayingGame)));
    }
}

#[derive(Resource, Reflect, InspectorOptions, Clone, Debug)]
#[reflect(Resource, InspectorOptions)]
pub struct Boundary {
    pub cell_count:               UVec3,
    pub color:                    Color,
    #[inspector(min = 0.0, max = 1.0, display = NumberDisplay::Slider)]
    pub distance_approach:        f32,
    #[inspector(min = 0.0, max = 1.0, display = NumberDisplay::Slider)]
    pub distance_shrink:          f32,
    #[inspector(min = 0.01, max = 10.0, display = NumberDisplay::Slider)]
    pub line_width:               f32,
    #[inspector(min = 50., max = 300., display = NumberDisplay::Slider)]
    pub scalar:                   f32,
    #[inspector(min = 1., max = 10., display = NumberDisplay::Slider)]
    pub smallest_teleport_circle: f32,
    pub transform:                Transform,
}

impl Default for Boundary {
    fn default() -> Self {
        let cell_count = UVec3::new(2, 1, 1);
        let scalar = 110.;

        Self {
            cell_count,
            color: Color::from(tailwind::BLUE_300),
            distance_approach: 0.5,
            distance_shrink: 0.25,
            line_width: 4.,
            scalar,
            smallest_teleport_circle: 5.,
            transform: Transform::from_scale(scalar * cell_count.as_vec3()),
        }
    }
}

#[derive(Default, Reflect, GizmoConfigGroup)]
pub struct BoundaryGizmos {}

fn update_gizmos_config(mut config_store: ResMut<GizmoConfigStore>, boundary_config: Res<Boundary>) {
    for (_, any_config, _) in config_store.iter_mut() {
        any_config.render_layers = RenderLayers::from_layers(RenderLayer::Game.layers());
        any_config.line_width = 2.;
    }

    // so we can avoid an error of borrowing the mutable config_store twice
    // in the same context
    {
        let (config, _) = config_store.config_mut::<BoundaryGizmos>();
        config.line_width = boundary_config.line_width;
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

    pub fn scale(&self) -> Vec3 { self.scalar * self.cell_count.as_vec3() }

    pub fn longest_diagonal(&self) -> f32 {
        let boundary_scale = self.scale();
        (boundary_scale.x.powi(2) + boundary_scale.y.powi(2) + boundary_scale.z.powi(2)).sqrt()
    }

    pub fn max_missile_distance(&self) -> f32 {
        let boundary_scale = self.scale();
        boundary_scale.x.max(boundary_scale.y).max(boundary_scale.z)
    }
    
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
        
        let mut t_min = f32::MAX;

        for (start, dir, pos_bound, neg_bound) in [
            (origin.x, direction.x, boundary_max.x, boundary_min.x),
            (origin.y, direction.y, boundary_max.y, boundary_min.y),
            (origin.z, direction.z, boundary_max.z, boundary_min.z),
        ] {
            if dir != 0.0 {
                let mut  update_t_min = |boundary: f32| {
                    let t = (boundary - start) / dir;
                    let point = origin + direction * t;
                    if t > 0.0
                        && t < t_min
                        && is_in_bounds(point, start, origin, boundary_min, boundary_max)
                    {
                        t_min = t;
                    }
                };

                update_t_min(pos_bound);
                update_t_min(neg_bound);
            }
        }

        if t_min != f32::MAX {
            let edge_point = origin + direction * t_min;
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

fn draw_boundary(mut boundary: ResMut<Boundary>, mut gizmos: Gizmos<BoundaryGizmos>) {
    // updating the boundary resource transform from its configuration so it can be
    // dynamically changed with the inspector while the game is running
    // the boundary transform is used both for position but also
    // so the fixed camera can be positioned based on the boundary scale
    boundary.transform.scale = boundary.scale();

    gizmos
        .grid_3d(
            boundary.transform.translation,
            Quat::IDENTITY,
            boundary.cell_count,
            Vec3::splat(boundary.scalar),
            boundary.color,
        )
        .outer_edges();
}
