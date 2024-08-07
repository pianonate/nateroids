use bevy::prelude::KeyCode::{
    ArrowDown, ArrowLeft, ArrowRight, ArrowUp, Escape, KeyA, KeyC, KeyD, KeyM, KeyS, KeyW,
    ShiftLeft, Space, F2, F3, F4,
};
use bevy::{prelude::MouseButton::Middle, prelude::*};
use leafwing_input_manager::prelude::*;

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app
            // camera will be added to the camera when it is spawned
            .add_plugins(InputManagerPlugin::<CameraMovement>::default())
            // global actions such as Pause added as a resource to be used wherever necessary
            .add_plugins(InputManagerPlugin::<GlobalAction>::default())
            // spaceship will have input attached to it when spawning a spaceship
            .add_plugins(InputManagerPlugin::<SpaceshipAction>::default())
            .init_resource::<ActionState<GlobalAction>>()
            // this map is available to all systems
            .insert_resource(GlobalAction::global_input_map())
            .init_resource::<ActionState<SpaceshipAction>>()
            // this map is available to all systems
            .insert_resource(SpaceshipAction::spaceship_input_map());
    }
}

#[derive(Clone, Debug, Copy, PartialEq, Eq, Hash, Reflect)]
pub enum CameraMovement {
    Orbit,
    Pan,
    Zoom,
}

impl CameraMovement {
    pub fn camera_input_map() -> InputMap<Self> {
        let pan_chord = ButtonlikeChord::new([ShiftLeft]).with(Middle);

        // this is my attempt to setup camera controls for a PanOrbit-style camera
        // a la the way blender works - it's a pain in the ass and it only works so so
        // todo: you could publish this as a crate if you wrap it up nicely with the Camera
        //       it might be something blender fans would like
        InputMap::default()
            // Orbit:  mouse wheel pressed with mouse move
            .with_dual_axis(
                CameraMovement::Orbit,
                DualAxislikeChord::new(Middle, MouseMove::default()),
            )
            // Orbit: scrolling on the trackpad
            .with_dual_axis(CameraMovement::Orbit, MouseScroll::default())
            // Pan: LeftShift plus scrolling on the trackpad
            .with_dual_axis(
                CameraMovement::Pan,
                DualAxislikeChord::new(ShiftLeft, MouseScroll::default()),
            )
            .with_dual_axis(
                CameraMovement::Pan,
                DualAxislikeChord::new(pan_chord, MouseScroll::default()),
            )
            // zoom: Mouse Scroll Wheel - Y axis
            .with_axis(CameraMovement::Zoom, MouseScrollAxis::Y)
    }
}

impl Actionlike for CameraMovement {
    fn input_control_kind(&self) -> InputControlKind {
        match self {
            CameraMovement::Orbit => InputControlKind::DualAxis,
            CameraMovement::Pan => InputControlKind::DualAxis,
            CameraMovement::Zoom => InputControlKind::Axis,
        }
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
