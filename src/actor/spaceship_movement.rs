use crate::input::{toggle_active, GlobalAction};
use bevy::prelude::*;
use bevy_inspector_egui::{
    inspector_options::std_options::NumberDisplay,
    prelude::*,
    quick::ResourceInspectorPlugin,
};

pub struct SpaceshipMovementPlugin;

impl Plugin for SpaceshipMovementPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<SpaceshipMovementConfig>()
            .add_plugins(
                ResourceInspectorPlugin::<SpaceshipMovementConfig>::default().run_if(
                    toggle_active(false, GlobalAction::SpaceshipMovementInspector),
                ),
            )
            .init_resource::<SpaceshipMovementConfig>();
    }
}

#[derive(Resource, Reflect, InspectorOptions, Debug, PartialEq, Clone, Copy)]
#[reflect(Resource, InspectorOptions)]
pub struct SpaceshipMovementConfig {
    #[inspector(min = 30., max = 300.0, display = NumberDisplay::Slider)]
    pub acceleration:   f32,
    #[inspector(min = 50., max = 300.0, display = NumberDisplay::Slider)]
    pub max_speed:      f32,
    #[inspector(min = 1.0, max = 10.0, display = NumberDisplay::Slider)]
    pub rotation_speed: f32,
}

impl Default for SpaceshipMovementConfig {
    fn default() -> Self {
        Self {
            acceleration:   60.,
            rotation_speed: 5.0,
            max_speed:      80.,
        }
    }
}
