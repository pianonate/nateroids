use bevy::prelude::{
    KeyCode::{
        ArrowDown,
        ArrowLeft,
        ArrowRight,
        ArrowUp,
        Escape,
        Home,
        KeyA,
        KeyB,
        KeyC,
        KeyD,
        KeyF,
        KeyL,
        KeyP,
        KeyS,
        KeyW,
        ShiftLeft,
        ShiftRight,
        Space,
        F1,
        F12,
        F2,
        F3,
    },
    MouseButton,
    *,
};
use leafwing_input_manager::prelude::*;

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app
            // camera will be added to the camera when it is spawned
            .add_plugins(InputManagerPlugin::<CameraMovement>::default())
            // spaceship will have input attached to it when spawning a spaceship
            .add_plugins(InputManagerPlugin::<SpaceshipAction>::default())
            .init_resource::<ActionState<SpaceshipAction>>()
            .insert_resource(SpaceshipAction::spaceship_input_map())
            // global actions such as Pause added as a resource to be used wherever
            .add_plugins(InputManagerPlugin::<GlobalAction>::default())
            .init_resource::<ActionState<GlobalAction>>()
            .insert_resource(GlobalAction::global_input_map());
    }
}

#[derive(Clone, Debug, Copy, PartialEq, Eq, Hash, Reflect)]
pub enum CameraMovement {
    Home,
    Orbit,
    Pan,
    Zoom,
}

impl CameraMovement {
    pub fn camera_input_map() -> InputMap<Self> {
        let pan_chord = ButtonlikeChord::new([ShiftLeft]).with(MouseButton::Middle);

        // this is my attempt to setup camera controls for a PanOrbit-style camera
        // a la the way blender works - it's a pain in the ass and it only works so so
        // todo: you could publish this as a crate if you wrap it up nicely with the
        // Camera       it might be something blender fans would like
        InputMap::default()
            // Orbit:  mouse wheel pressed with mouse move
            .with(CameraMovement::Home, Home)
            .with(CameraMovement::Home, F12)
            .with_dual_axis(
                CameraMovement::Orbit,
                DualAxislikeChord::new(MouseButton::Middle, MouseMove::default()),
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
            // you could pan with left mouse click if this was enabled...
            // todo: #bevyquestion - how can we stop egui from passing mouse events through to the
            // main game? .with_dual_axis(
            //     CameraMovement::Pan,
            //     DualAxislikeChord::new(MouseButton::Left, MouseMove::default()),
            // )
            // zoom: Mouse Scroll Wheel - Y axis
            .with_axis(CameraMovement::Zoom, MouseScrollAxis::Y)
    }
}

impl Actionlike for CameraMovement {
    fn input_control_kind(&self) -> InputControlKind {
        match self {
            CameraMovement::Home => InputControlKind::Button,
            CameraMovement::Orbit => InputControlKind::DualAxis,
            CameraMovement::Pan => InputControlKind::DualAxis,
            CameraMovement::Zoom => InputControlKind::Axis,
        }
    }
}

// This is the list of "things I want the spaceship to be able to do based on
// input"
#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug, Reflect)]
pub enum SpaceshipAction {
    Accelerate,
    ContinuousFire,
    Decelerate,
    Fire,
    TurnLeft,
    TurnRight,
}

// #todo #bug - i can't use Shift-C as it invokes ContinuousFire even thought
// the              ClashStrategy::PrioritizeLongest is on by default (and i
// tried explicitly)
impl SpaceshipAction {
    pub fn spaceship_input_map() -> InputMap<Self> {
        let mut input_map = InputMap::default();

        input_map.insert(Self::Accelerate, KeyW);
        input_map.insert(Self::Accelerate, ArrowUp);

        input_map.insert(Self::ContinuousFire, KeyF);

        input_map.insert(Self::Decelerate, KeyS);
        input_map.insert(Self::Decelerate, ArrowDown);

        input_map.insert(Self::Fire, Space);

        input_map.insert(Self::TurnLeft, KeyA);
        input_map.insert(Self::TurnLeft, ArrowLeft);

        input_map.insert(Self::TurnRight, KeyD);
        input_map.insert(Self::TurnRight, ArrowRight);

        input_map
    }
}

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug, Reflect)]
pub enum GlobalAction {
    AABBs,
    BoundaryInspector,
    CameraInspector,
    Debug,
    LightsInspector,
    MissileInspector,
    NateroidInspector,
    Physics,
    PlanesInspector,
    Pause,
    SpaceshipInspector,
    SpaceshipMovementInspector,
    Stars,
}

/// Use Debug like this - invoke it with a system as follows:
/// ```rust
/// app.add_systems(Update, my_debug_system.run_if(toggle_active(false, GlobalAction::Debug))
/// ```
/// useful when you want to limit the amount of info that is being emitted
///
/// similarly you can also ask for the GlobalAction and use it in your code
/// directly ```rust
/// fn my_system(user_input: Res<ActionState<GlobalAction>>) {
///    if user_input.pressed(&GlobalAction::Debug) {
///       // whatever debug statements you're using will only happen while you
/// press it    }
/// }
/// ```
impl GlobalAction {
    //todo: #bug - do this with an enum to make sure you cover all types
    pub fn global_input_map() -> InputMap<Self> {
        let mut input_map = InputMap::default();

        let create_dual_input =
            |action: GlobalAction, key: KeyCode, input_map: &mut InputMap<GlobalAction>| {
                input_map.insert(action, ButtonlikeChord::new([ShiftLeft]).with(key));
                input_map.insert(action, ButtonlikeChord::new([ShiftRight]).with(key));
            };

        input_map.insert(Self::AABBs, F1);
        create_dual_input(Self::BoundaryInspector, KeyB, &mut input_map);
        create_dual_input(Self::CameraInspector, KeyC, &mut input_map);
        create_dual_input(Self::Debug, KeyD, &mut input_map);
        create_dual_input(Self::LightsInspector, KeyL, &mut input_map);
        create_dual_input(Self::MissileInspector, KeyCode::Digit1, &mut input_map);
        create_dual_input(Self::NateroidInspector, KeyCode::Digit2, &mut input_map);
        input_map.insert(Self::Pause, Escape);
        create_dual_input(Self::PlanesInspector, KeyP, &mut input_map);
        input_map.insert(Self::Physics, F2);
        create_dual_input(Self::SpaceshipInspector, KeyCode::Digit3, &mut input_map);
        create_dual_input(
            Self::SpaceshipMovementInspector,
            KeyCode::Digit4,
            &mut input_map,
        );
        input_map.insert(Self::Stars, F3);

        input_map
    }
}

#[derive(Default)]
pub struct ToggleState {
    pub state: bool,
}

// todo: #doc document how to use toggle action and why it's cool
// i couldn't have made this without gpt help - here's what it's telling me
//
// Each use of toggle_active() gets its own Local<ToggleState>.
//
// The Res<ActionState<GlobalAction>> is shared across the app, but each closure
// gets its own reference to it.
//
// Bevy's dependency injection system automatically provides these resources
// when the closure is executed, based on the types specified in the closure's
// signature.
//
// the impl Fn(..) piece is the key in that we're telling rust
// that "this function returns some type that implements the Fn(...) trait".
// so instead of a concrete type, we're specifying a trait that the
// returned type implements
// rust infers the actual concrete type based on the function body - in this
// case, a closure
// so:  toggle_active takes normal args and returns
//      * something that is a function (impl Fn)
//      * takes these two other params that bevy can dependency inject just like
//        systems
//      * returns a bool
//
// this is crazy to me
pub fn toggle_active(
    default: bool,
    action: GlobalAction,
) -> impl Fn(Res<ActionState<GlobalAction>>, Local<ToggleState>) -> bool {
    move |action_state: Res<ActionState<GlobalAction>>,
          mut state: Local<ToggleState>| {
        if action_state.just_pressed(&action) {
            state.state = !state.state;
        }

        if state.state {
            !default
        } else {
            default
        }
    }
}
