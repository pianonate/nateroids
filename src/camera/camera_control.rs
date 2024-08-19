use crate::global_input::{
    toggle_active,
    GlobalAction,
};

use bevy::{
    color::{
        palettes::tailwind,
        Color,
    },
    prelude::*,
};
use bevy_inspector_egui::{
    inspector_options::std_options::NumberDisplay,
    prelude::*,
    quick::ResourceInspectorPlugin,
};
use leafwing_input_manager::{
    input_map::InputMap,
    plugin::InputManagerPlugin,
    prelude::{
        ButtonlikeChord,
        DualAxislikeChord,
        MouseMove,
        MouseScroll,
        MouseScrollAxis,
    },
    Actionlike,
    InputControlKind,
};
use strum::{
    EnumIter,
    IntoEnumIterator,
};

pub struct CameraControlPlugin;

impl Plugin for CameraControlPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<CameraConfig>()
            .add_plugins(
                ResourceInspectorPlugin::<CameraConfig>::default()
                    .run_if(toggle_active(false, GlobalAction::CameraInspector)),
            )
            .init_resource::<CameraConfig>()
            .add_plugins(InputManagerPlugin::<CameraControl>::default());
    }
}

#[derive(Resource, Reflect, InspectorOptions, Debug, PartialEq, Clone, Copy)]
#[reflect(Resource, InspectorOptions)]
pub struct CameraConfig {
    pub clear_color:               Color,
    #[inspector(min = 0.0, max = 1.0, display = NumberDisplay::Slider)]
    pub darkening_factor:          f32,
    #[inspector(min = 0.0, max = 1.0, display = NumberDisplay::Slider)]
    pub bloom_intensity:           f32,
    #[inspector(min = 0.0, max = 1.0, display = NumberDisplay::Slider)]
    pub bloom_low_frequency_boost: f32,
    #[inspector(min = 0.0, max = 1.0, display = NumberDisplay::Slider)]
    pub bloom_high_pass_frequency: f32,
    #[inspector(min = 0.0, max = 1.0, display = NumberDisplay::Slider)]
    pub orbit_speed:               f32,
    #[inspector(min = 10.0, max = 200.0, display = NumberDisplay::Slider)]
    pub zoom_sensitivity_pinch:    f32,
    #[inspector(min = 1.0, max = 20.0, display = NumberDisplay::Slider)]
    pub zoom_sensitivity_mouse:    f32,
}

impl Default for CameraConfig {
    fn default() -> Self {
        Self {
            clear_color:               Color::from(tailwind::SLATE_900),
            darkening_factor:          0.002,
            bloom_intensity:           0.9,
            bloom_low_frequency_boost: 0.5,
            bloom_high_pass_frequency: 0.5,
            orbit_speed:               0.01,
            zoom_sensitivity_pinch:    100.,
            zoom_sensitivity_mouse:    5.,
        }
    }
}

// this is my attempt to setup camera controls for a PanOrbit-style camera
// a la the way blender works - it's a pain in the ass and it only works so so
// todo: you could publish this as a crate if you wrap it up nicely with the
//       Camera it might be something blender fans would like
#[derive(Clone, Debug, EnumIter, Copy, PartialEq, Eq, Hash, Reflect)]
pub enum CameraControl {
    Home,
    Orbit,
    Pan,
    Zoom,
}

impl Actionlike for CameraControl {
    fn input_control_kind(&self) -> InputControlKind {
        match self {
            CameraControl::Home => InputControlKind::Button,
            CameraControl::Orbit => InputControlKind::DualAxis,
            CameraControl::Pan => InputControlKind::DualAxis,
            CameraControl::Zoom => InputControlKind::Axis,
        }
    }
}

// you could pan with left mouse click if this was enabled...
// todo: #bevyquestion #bug - how can we stop egui from passing mouse events
// through to the       main game?
// .with_dual_axis(
//     CameraMovement::Pan,
//     DualAxislikeChord::new(MouseButton::Left, MouseMove::default()),
// )
impl CameraControl {
    pub fn camera_input_map() -> InputMap<Self> {
        Self::iter().fold(InputMap::default(), |input_map, action| match action {
            Self::Home => input_map.with_one_to_many(action, [KeyCode::Home, KeyCode::F12]),
            Self::Orbit => input_map
                .with_dual_axis(
                    action,
                    DualAxislikeChord::new(MouseButton::Middle, MouseMove::default()),
                )
                .with_dual_axis(action, MouseScroll::default()),
            Self::Pan => input_map
                .with_dual_axis(
                    action,
                    DualAxislikeChord::new(KeyCode::ShiftLeft, MouseScroll::default()),
                )
                .with_dual_axis(
                    action,
                    DualAxislikeChord::new(
                        ButtonlikeChord::new([KeyCode::ShiftLeft]).with(MouseButton::Middle),
                        MouseScroll::default(),
                    ),
                ),
            Self::Zoom => input_map.with_axis(action, MouseScrollAxis::Y),
        })
    }
}
