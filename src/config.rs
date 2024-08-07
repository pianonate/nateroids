use crate::stars::GAME_LAYER;
use bevy::prelude::*;
use bevy::render::view::RenderLayers;
use bevy_inspector_egui::InspectorOptions;

pub struct ConfigPlugin;

impl Plugin for ConfigPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameConfig>()
            .init_gizmo_group::<BoundaryGizmos>()
            .add_systems(Startup, init_gizmo_configs);
    }
}

#[derive(Default, Reflect, GizmoConfigGroup)]
pub struct BoundaryGizmos {}

fn init_gizmo_configs(mut config_store: ResMut<GizmoConfigStore>) {
    for (_, any_config, _) in config_store.iter_mut() {
        any_config.render_layers = RenderLayers::layer(GAME_LAYER);
        any_config.line_width = 1.;
    }

    // so we can avoid an error of borrowing the mutable config_store twice
    // in the same context
    {
        let (config, _) = config_store.config_mut::<BoundaryGizmos>();
        config.line_width = 5.;
    }
}

// centralize scale defaults
// plus this allows us to use the inspector to dynamically change them
// to try out different ratios while the game is running
#[derive(Debug, Clone, Reflect, Resource, InspectorOptions)]
#[reflect(Resource)]
pub struct GameConfig {
    pub boundary_cell_scalar: f32,
    pub missile_sphere_radius: f32,
    pub star_count: usize,
    pub star_radius: f32,
    pub star_field_inner_diameter: f32,
    pub star_field_outer_diameter: f32,
    pub missile: ColliderConstant,
    pub nateroid: ColliderConstant,
    pub spaceship: ColliderConstant,
}

#[derive(Debug, Clone, Reflect, Resource, InspectorOptions)]
#[reflect(Resource)]
pub struct ColliderConstant {
    pub radius: f32,
    pub scalar: f32,
    pub spawnable: bool,
    pub velocity: f32,
}

impl Default for GameConfig {
    fn default() -> Self {
        // these scales were set by eye-balling the game
        // if you get different assets these will likely need to change
        // to match the assets size
        Self {
            boundary_cell_scalar: 110.,
            missile_sphere_radius: 2.,
            star_count: 5000,
            star_radius: 5.,
            star_field_inner_diameter: 1000.,
            star_field_outer_diameter: 20000.,
            missile: ColliderConstant {
                radius: 0.5,
                scalar: 1.5,
                spawnable: true,
                velocity: 75.,
            },
            nateroid: ColliderConstant {
                radius: 2.3,
                scalar: 2.,
                spawnable: true,
                velocity: 30.,
            },
            spaceship: ColliderConstant {
                radius: 6.25,
                scalar: 0.8,
                spawnable: true,
                velocity: 40.,
            },
        }
    }
}
