use bevy::prelude::KeyCode::{ArrowDown, ArrowLeft, ArrowRight, ArrowUp, KeyA, KeyD, KeyS, KeyW};
use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(InputManagerPlugin::<Action>::default());
    }
}

// This is the list of "things I want the spaceship to be able to do based on input"
#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug, Reflect)]
pub enum Action {
    Accelerate,
    Decelerate,
    Fire,
    TurnLeft,
    TurnRight,
}

impl Action {
    pub(crate) fn default_input_map() -> InputMap<Self> {
        let mut input_map = InputMap::default();

        input_map.insert(Self::Accelerate, KeyW);
        input_map.insert(Self::Accelerate, ArrowUp);

        input_map.insert(Self::Decelerate, KeyS);
        input_map.insert(Self::Decelerate, ArrowDown);

        input_map.insert(Self::Fire, KeyCode::Space);

        input_map.insert(Self::TurnLeft, KeyA);
        input_map.insert(Self::TurnLeft, ArrowLeft);

        input_map.insert(Self::TurnRight, KeyD);
        input_map.insert(Self::TurnRight, ArrowRight);

        input_map
    }
}
