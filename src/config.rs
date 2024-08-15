use crate::camera::RenderLayer;
use bevy::{
    color::{
        palettes::{
            css,
            tailwind,
        },
        Color::Srgba,
    },
    prelude::*,
    render::view::RenderLayers,
};

pub struct ConfigPlugin;

impl Plugin for ConfigPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AppearanceConfig>()
            .init_gizmo_group::<BoundaryGizmos>()
            .add_systems(Startup, init_gizmo_configs);
    }
}

#[derive(Default, Reflect, GizmoConfigGroup)]
pub struct BoundaryGizmos {}

fn init_gizmo_configs(
    mut config_store: ResMut<GizmoConfigStore>,
    appearance_config: Res<AppearanceConfig>,
) {
    for (_, any_config, _) in config_store.iter_mut() {
        any_config.render_layers = RenderLayers::from_layers(RenderLayer::Game.layers());
        any_config.line_width = 2.;
    }

    // so we can avoid an error of borrowing the mutable config_store twice
    // in the same context
    {
        let (config, _) = config_store.config_mut::<BoundaryGizmos>();
        config.line_width = appearance_config.boundary_line_width;
    }
}

#[derive(Resource, Reflect, Debug)]
#[reflect(Resource)]
pub struct AppearanceConfig {
    pub bloom_intensity:                f32,
    pub bloom_low_frequency_boost:      f32,
    pub bloom_high_pass_frequency:      f32,
    pub boundary_color:                 Color,
    pub boundary_distance_approach:     f32,
    pub boundary_distance_shrink:       f32,
    pub boundary_line_width:            f32,
    pub boundary_cell_count:            UVec3,
    pub boundary_scalar:                f32,
    pub clear_color:                    Color,
    pub clear_color_darkening_factor:   f32,
    pub missile_forward_spawn_distance: f32,
    pub smallest_teleport_circle:       f32,
    pub splash_timer:                   f32,
    pub zoom_sensitivity_pinch:         f32,
    pub zoom_sensitivity_mouse:         f32,
}

// centralize appearance defaults
// plus this allows us to use the inspector to dynamically change them
// to try out different ratios while the game is running
impl Default for AppearanceConfig {
    fn default() -> Self {
        Self {
            bloom_intensity:                0.9,
            bloom_low_frequency_boost:      0.5,
            bloom_high_pass_frequency:      0.5,
            boundary_color:                 Color::from(tailwind::BLUE_300),
            boundary_distance_approach:     0.5,
            boundary_distance_shrink:       0.25,
            boundary_line_width:            4.,
            boundary_cell_count:            UVec3::new(2, 1, 1),
            boundary_scalar:                110.,
            clear_color:                    Srgba(css::MIDNIGHT_BLUE),
            clear_color_darkening_factor:   0.019,
            missile_forward_spawn_distance: 5.6,
            smallest_teleport_circle:       5.,
            splash_timer:                   2.,
            zoom_sensitivity_pinch:         100.,
            zoom_sensitivity_mouse:         5.,
        }
    }
}
