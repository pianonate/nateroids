use bevy::prelude::*;
use bevy_inspector_egui::InspectorOptions;

pub struct GameScalePlugin;

impl Plugin for GameScalePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameScale>();
    }
}

// centralize scale defaults
// plus this allows us to use the inspector to dynamically change them
// to try out different ratios while the game is running
#[derive(Debug, Clone, Reflect, Resource, InspectorOptions)]
#[reflect(Resource)]
pub struct GameScale {
    pub boundary_cell_scalar: f32,
    pub star_count: usize,
    pub star_scale: f32,
    pub star_field_inner_diameter: f32,
    pub(crate) star_field_outer_diameter: f32,
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

impl Default for GameScale {
    fn default() -> Self {
        // these scales were set by eye-balling the game
        // if you get different assets these will likely need to change
        // to match the assets size
        Self {
            boundary_cell_scalar: 110.,
            star_count: 10000,
            star_scale: 1.,
            star_field_inner_diameter: 100.,
            star_field_outer_diameter: 10000.,
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
