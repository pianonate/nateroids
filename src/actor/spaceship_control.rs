use crate::input::{
    toggle_active,
    GlobalAction,
};
use bevy::prelude::*;
use bevy_inspector_egui::{
    inspector_options::std_options::NumberDisplay,
    prelude::*,
    quick::ResourceInspectorPlugin,
};
use leafwing_input_manager::{
    action_state::ActionState,
    input_map::InputMap,
    plugin::InputManagerPlugin,
    Actionlike,
};
use strum::{
    EnumIter,
    IntoEnumIterator,
};

pub struct SpaceshipControlPlugin;

impl Plugin for SpaceshipControlPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<SpaceshipControlConfig>()
            .add_plugins(
                ResourceInspectorPlugin::<SpaceshipControlConfig>::default().run_if(
                    toggle_active(false, GlobalAction::SpaceshipControlInspector),
                ),
            )
            .init_resource::<SpaceshipControlConfig>()
            // spaceship will have input attached to it when spawning a spaceship
            .add_plugins(InputManagerPlugin::<SpaceshipControl>::default())
            .init_resource::<ActionState<SpaceshipControl>>()
            .insert_resource(SpaceshipControl::generate_input_map());
    }
}

#[derive(Resource, Reflect, InspectorOptions, Debug, PartialEq, Clone, Copy)]
#[reflect(Resource, InspectorOptions)]
pub struct SpaceshipControlConfig {
    #[inspector(min = 30., max = 300.0, display = NumberDisplay::Slider)]
    pub acceleration:   f32,
    #[inspector(min = 50., max = 300.0, display = NumberDisplay::Slider)]
    pub max_speed:      f32,
    #[inspector(min = 1.0, max = 10.0, display = NumberDisplay::Slider)]
    pub rotation_speed: f32,
}

impl Default for SpaceshipControlConfig {
    fn default() -> Self {
        Self {
            acceleration:   60.,
            rotation_speed: 5.0,
            max_speed:      80.,
        }
    }
}

// This is the list of "things I want the spaceship to be able to do based on
// input"
#[derive(Actionlike, EnumIter, PartialEq, Eq, Clone, Copy, Hash, Debug, Reflect)]
pub enum SpaceshipControl {
    Accelerate,
    ContinuousFire,
    Fire,
    TurnLeft,
    TurnRight,
}

// #todo #bug - i can't use Shift-C as it invokes ContinuousFire even thought
//               the ClashStrategy::PrioritizeLongest is on by default (and i
//              tried explicitly)
impl SpaceshipControl {
    pub fn generate_input_map() -> InputMap<Self> {
        Self::iter().fold(InputMap::default(), |mut input_map, action| {
            match action {
                Self::Accelerate => {
                    input_map.insert(action, KeyCode::KeyW);
                    input_map.insert(action, KeyCode::ArrowUp);
                },
                Self::TurnLeft => {
                    input_map.insert(action, KeyCode::KeyA);
                    input_map.insert(action, KeyCode::ArrowLeft);
                },
                Self::TurnRight => {
                    input_map.insert(action, KeyCode::KeyD);
                    input_map.insert(action, KeyCode::ArrowRight);
                },
                Self::Fire => {
                    input_map.insert(action, KeyCode::Space);
                },
                Self::ContinuousFire => {
                    input_map.insert(action, KeyCode::KeyF);
                },
            }
            input_map
        })
    }
}
