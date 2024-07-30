#[cfg(debug_assertions)]
use crate::schedule::InGameSet;

#[cfg(debug_assertions)]
use bevy_inspector_egui::{prelude::*, InspectorOptions};

use bevy::prelude::KeyCode::{
    ArrowDown, ArrowLeft, ArrowRight, ArrowUp, Escape, KeyA, KeyC, KeyD, KeyM, KeyS, KeyW, Space,
    F2, F3, F4,
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

        #[cfg(debug_assertions)]
        app
            // puts us in debug mode which can be checked anywhere
            .init_resource::<DebugMode>()
            .init_resource::<InspectorMode>()
            .add_systems(Update, toggle_debug.in_set(InGameSet::UserInput))
            .add_systems(Update, toggle_inspector.in_set(InGameSet::UserInput));
    }
}

// the default bool is false, which is what we want
#[cfg(debug_assertions)]
#[derive(Reflect, Resource, Default, InspectorOptions)]
#[reflect(InspectorOptions)]
pub struct DebugMode {
    pub enabled: bool,
}

#[cfg(debug_assertions)]
fn toggle_debug(user_input: Res<ActionState<GlobalAction>>, mut debug_mode: ResMut<DebugMode>) {
    if user_input.just_pressed(&GlobalAction::Debug) {
        debug_mode.enabled = !debug_mode.enabled;
        println!("DebugMode: {}", debug_mode.enabled);
    }
}

#[cfg(debug_assertions)]
#[derive(Resource, Debug, Default)]
pub struct InspectorMode {
    pub enabled: bool,
}

#[cfg(debug_assertions)]
pub fn inspector_mode_enabled(inspector_mode: Res<InspectorMode>) -> bool {
    inspector_mode.enabled
}
#[cfg(debug_assertions)]
fn toggle_inspector(
    user_input: Res<ActionState<GlobalAction>>,
    mut inspector_mode: ResMut<InspectorMode>,
) {
    if user_input.just_pressed(&GlobalAction::Inspector) {
        inspector_mode.enabled = !inspector_mode.enabled;
        println!("InspectorMode: {}", inspector_mode.enabled);
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
    Inspector,
    Pause,
}

/// Use Debug like this - pull it into a system as follows:
/// ```rust
/// fn some_system(
///    debug: Res<DebugEnabled>,
/// )
/// ```
/// DebugEnabled is a simple tuple struct with a boolean so the first (.0) parameter
/// tells you if it's enabled or not
/// ```rust
///    if debug.enabled() {
///       println!("Debug action was just pressed!");
///    }
/// ```
impl GlobalAction {
    pub fn global_input_map() -> InputMap<Self> {
        let mut input_map = InputMap::default();
        input_map.insert(Self::Debug, F2);
        input_map.insert(Self::Diagnostics, F3);
        input_map.insert(Self::Inspector, F4);
        input_map.insert(Self::Pause, Escape);
        input_map
    }
}
