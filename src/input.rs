use bevy::prelude::*;
use leafwing_input_manager::prelude::*;
use strum::{
    EnumIter,
    IntoEnumIterator,
};

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app
            // camera will be added to the camera when it is spawned
            .add_plugins(InputManagerPlugin::<CameraMovement>::default())
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
        let pan_chord = ButtonlikeChord::new([KeyCode::ShiftLeft]).with(MouseButton::Middle);

        // this is my attempt to setup camera controls for a PanOrbit-style camera
        // a la the way blender works - it's a pain in the ass and it only works so so
        // todo: you could publish this as a crate if you wrap it up nicely with the
        // Camera       it might be something blender fans would like
        InputMap::default()
            // Orbit:  mouse wheel pressed with mouse move
            .with(CameraMovement::Home, KeyCode::Home)
            .with(CameraMovement::Home, KeyCode::F12)
            .with_dual_axis(
                CameraMovement::Orbit,
                DualAxislikeChord::new(MouseButton::Middle, MouseMove::default()),
            )
            // Orbit: scrolling on the trackpad
            .with_dual_axis(CameraMovement::Orbit, MouseScroll::default())
            // Pan: LeftShift plus scrolling on the trackpad
            .with_dual_axis(
                CameraMovement::Pan,
                DualAxislikeChord::new(KeyCode::ShiftLeft, MouseScroll::default()),
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

#[derive(Actionlike, EnumIter, Reflect, PartialEq, Eq, Clone, Copy, Hash, Debug)]
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
    SpaceshipControlInspector,
    Stars,
}

/// GlobalActions assign keys to do a lot of obvious stuff. Debug is less
/// obvious.
///
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
    pub fn global_input_map() -> InputMap<Self> {
        fn insert_dual_input(map: &mut InputMap<GlobalAction>, action: GlobalAction, key: KeyCode) {
            map.insert(action, ButtonlikeChord::new([KeyCode::ShiftLeft]).with(key));
            map.insert(
                action,
                ButtonlikeChord::new([KeyCode::ShiftRight]).with(key),
            );
        }

        // while fold accumulates each pass - we just do an insert each time as a
        // statement and then return the map at the end of each iteration so the
        // accumulation works
        Self::iter().fold(InputMap::default(), |mut map, action| {
            match action {
                Self::AABBs => {
                    map.insert(action, KeyCode::F1);
                },
                Self::BoundaryInspector => {
                    insert_dual_input(&mut map, action, KeyCode::KeyB);
                },
                Self::CameraInspector => {
                    insert_dual_input(&mut map, action, KeyCode::KeyC);
                },
                Self::Debug => {
                    insert_dual_input(&mut map, action, KeyCode::KeyD);
                },
                Self::LightsInspector => {
                    insert_dual_input(&mut map, action, KeyCode::KeyL);
                },
                Self::MissileInspector => {
                    insert_dual_input(&mut map, action, KeyCode::Digit1);
                },
                Self::NateroidInspector => {
                    insert_dual_input(&mut map, action, KeyCode::Digit2);
                },
                Self::Physics => {
                    map.insert(action, KeyCode::F2);
                },
                Self::PlanesInspector => {
                    insert_dual_input(&mut map, action, KeyCode::KeyP);
                },
                Self::Pause => {
                    map.insert(action, KeyCode::Escape);
                },
                Self::SpaceshipInspector => {
                    insert_dual_input(&mut map, action, KeyCode::Digit3);
                },
                Self::SpaceshipControlInspector => {
                    insert_dual_input(&mut map, action, KeyCode::Digit4);
                },
                Self::Stars => {
                    map.insert(action, KeyCode::F3);
                },
            }
            map
        })
    }
}

// #todo: #bevyquestion #rustquestion - how does bevy know how to do the
// dependency injection with this impl?        because it makes using
// toggle_active super intuitive and useful

/// ToggleActive allows us to do something cool - we can use it like the bevy
/// input_toggle_active but it works with leafwing_input_manager input_map
/// entries so we can have simple syntax for toggling systems as a run condition
/// as follows:
///
/// ```
/// .add_systems(Update, my_system.run_if(toggle_active(false, GlobalAction::AABBs)))
/// ```
/// cool, huh? the fact that the closure works with Bevy's dependency injection
/// is rocket science to me- i don't know how it knows to do this but it does
pub fn toggle_active(
    default: bool,
    action: GlobalAction,
) -> impl Fn(Res<ActionState<GlobalAction>>, Local<ToggleState>) -> bool {
    move |action_state: Res<ActionState<GlobalAction>>, mut state: Local<ToggleState>| {
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

#[derive(Default)]
pub struct ToggleState {
    pub state: bool,
}
