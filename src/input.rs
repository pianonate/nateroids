use bevy::prelude::KeyCode::{
    ArrowDown, ArrowLeft, ArrowRight, ArrowUp, Escape, KeyA, KeyC, KeyD, KeyM, KeyS, KeyW,
    ShiftLeft, ShiftRight, Space, F12,
};
use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app
            // spaceship will have input attached to it when spawning a spaceship
            .add_plugins(InputManagerPlugin::<SpaceshipAction>::default())
            // global actions such as Pause added as a resource to be used wherever necessary
            .add_plugins(InputManagerPlugin::<GlobalAction>::default())
            .init_resource::<ActionState<GlobalAction>>()
            .insert_resource(GlobalAction::global_input_map());
    }
}

// This is the list of "things I want the spaceship to be able to do based on input"
#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug, Reflect)]
pub enum SpaceshipAction {
    Accelerate,
    ContinuousFire,
    Decelerate,
    Fire,
    MissileParty,
    TurnLeft,
    TurnRight,
}

impl SpaceshipAction {
    pub fn spaceship_input_map() -> InputMap<Self> {
        let mut input_map = InputMap::default();

        input_map.insert(Self::Accelerate, KeyW);
        input_map.insert(Self::Accelerate, ArrowUp);

        input_map.insert(Self::ContinuousFire, KeyC);

        input_map.insert(Self::Decelerate, KeyS);
        input_map.insert(Self::Decelerate, ArrowDown);

        input_map.insert(Self::Fire, Space);

        input_map.insert(Self::MissileParty, KeyM);

        input_map.insert(Self::TurnLeft, KeyA);
        input_map.insert(Self::TurnLeft, ArrowLeft);

        input_map.insert(Self::TurnRight, KeyD);
        input_map.insert(Self::TurnRight, ArrowRight);

        input_map
    }
}

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug, Reflect)]
pub enum GlobalAction {
    Diagnostics,
    Debug,
    Pause,
}

/// Use Debug like this - pull it into a system as follows:
/// ```rust
/// fn some_system(
///     user_input: Res<ActionState<GlobalAction>>,
/// )
/// ```
/// Then check if the Debug action was just pressed and you can use it in your system:
/// ```rust
///    if user_input.just_pressed(&GlobalAction::Debug) {
///       println!("Debug action was just pressed!");
///    }
/// ```
/// Then just press and the debug combo and the println will only print while you're holding it down
impl GlobalAction {
    pub fn global_input_map() -> InputMap<Self> {
        let mut input_map = InputMap::default();
        input_map.insert(Self::Debug, UserInput::chord([ShiftLeft, KeyD]));
        input_map.insert(Self::Debug, UserInput::chord([ShiftRight, KeyD]));
        input_map.insert(Self::Diagnostics, F12);
        input_map.insert(Self::Pause, Escape);
        input_map
    }
}