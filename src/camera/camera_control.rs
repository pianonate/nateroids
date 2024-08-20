use crate::{
    camera::PrimaryCamera,
    global_input::{
        toggle_active,
        GlobalAction,
    },
    orientation::CameraOrientation,
    state::{
        GameState,
        IsInspecting,
    },
};
use bevy::{
    color::palettes::tailwind,
    input::{
        gestures::PinchGesture,
        mouse::{
            MouseScrollUnit,
            MouseWheel,
        },
    },
    prelude::*,
};
use bevy_inspector_egui::{
    bevy_egui::EguiContext,
    inspector_options::std_options::NumberDisplay,
    prelude::*,
    quick::ResourceInspectorPlugin,
};
use leafwing_input_manager::prelude::*;
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
                    .run_if(toggle_active(false, GlobalAction::CameraConfigInspector)),
            )
            .init_resource::<CameraConfig>()
            .add_plugins(InputManagerPlugin::<CameraControl>::default())
            .add_systems(Update, check_inspector_state)
            .add_systems(
                Update,
                (
                    // order matters because we hack around the input manager
                    // that doesn't yet support trackpads
                    home_camera,
                    pinch_to_zoom,
                    zoom_camera,
                    orbit_camera,
                    pan_camera,
                )
                    .chain()
                    .run_if(in_state(IsInspecting::NotInspecting)),
            );
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
                // simulates blender pan on macOS touchpad
                .with_dual_axis(
                    action,
                    DualAxislikeChord::new(KeyCode::ShiftLeft, MouseScroll::default()),
                )
                // simulates blender pan with mouse middle wheel and ShiftLeft
                .with_dual_axis(
                    action,
                    DualAxislikeChord::new(
                        ButtonlikeChord::new([KeyCode::ShiftLeft]).with(MouseButton::Middle),
                        MouseScroll::default(),
                    ),
                )
                // typical pan by just click and drag
                .with_dual_axis(
                    action,
                    DualAxislikeChord::new(MouseButton::Left, MouseMove::default()),
                ),
            Self::Zoom => input_map.with_axis(action, MouseScrollAxis::Y),
        })
    }
}

fn check_inspector_state(
    mut contexts: Query<&mut EguiContext>,
    mut next_state: ResMut<NextState<GameState>>,
    state: Res<State<GameState>>,
) {
    if let Ok(mut context) = contexts.get_single_mut() {
        let ctx = context.get_mut();

        // Check if the mouse is over any egui area
        let is_mouse_over_inspector = ctx.is_pointer_over_area();
        
        let dragged =  ctx.dragged_id().is_some();
        
        let is_inspecting = is_mouse_over_inspector || dragged;
        
        if let GameState::InGame { paused, inspecting } = state.get() {
            if *inspecting != is_inspecting {
                next_state.set(GameState::InGame {
                    paused:     *paused,
                    inspecting: is_mouse_over_inspector,
                });
            }
        }
    }
}

fn home_camera(
    orientation: Res<CameraOrientation>,
    mut camera_transform: Query<(&mut Transform, &ActionState<CameraControl>), With<PrimaryCamera>>,
) {
    if let Ok((mut transform, action_state)) = camera_transform.get_single_mut() {
        if action_state.just_pressed(&CameraControl::Home) {
            *transform = orientation.config.locus;
        }
    }
}

fn pinch_to_zoom(
    mut query: Query<&mut Transform, With<PrimaryCamera>>,
    mut pinch_gesture_events: EventReader<PinchGesture>,
    config: Res<CameraConfig>,
) {
    for event in pinch_gesture_events.read() {
        if let Ok(mut transform) = query.get_single_mut() {
            impl_zoom(config.zoom_sensitivity_pinch, &mut transform, event.0);
        }
    }
}

fn zoom_camera(
    mut query: Query<(&mut Transform, &mut ActionState<CameraControl>), With<PrimaryCamera>>,
    mut mouse_wheel_events: EventReader<MouseWheel>,
    config: Res<CameraConfig>,
) {
    if let Ok((mut transform, mut action_state)) = query.get_single_mut() {
        let zoom_delta = match should_zoom(&mut mouse_wheel_events, &mut action_state) {
            Some(value) => value,
            None => return,
        };

        impl_zoom(config.zoom_sensitivity_mouse, &mut transform, zoom_delta);

        // cleanup any dual_axis propagating from orbit so that Pan doesn't see it
        elide_dual_axis_data(&mut action_state);
    }
}

fn impl_zoom(sensitivity: f32, transform: &mut Mut<Transform>, zoom_delta: f32) {
    // Calculate zoom direction based on camera's current orientation
    let zoom_direction = transform.forward();

    // Calculate zoom amount
    let zoom_speed = sensitivity; // Adjust this value to control zoom sensitivity
    let zoom_amount = zoom_delta * zoom_speed;

    // Apply zoom
    transform.translation += zoom_direction * zoom_amount;
}

// does a lot of stuff! determines if we're mouse or trackpad (where zooming is
// not allowed in order to match blender behavior) it also cleans up after what
// i consider to be a leafwing issue. extracts the zoom amount and then returns
// an Option of it
fn should_zoom(
    mouse_wheel_events: &mut EventReader<MouseWheel>,
    action_state: &mut Mut<ActionState<CameraControl>>,
) -> Option<f32> {
    let mut trackpad = false;

    // hack to determine if the input was from a mouse or a trackpad
    for event in mouse_wheel_events.read() {
        trackpad = match event.unit {
            MouseScrollUnit::Line => false,
            MouseScrollUnit::Pixel => true,
        };
    }

    // leafwing doesn't yet allow us to distinguish between mouse input and trackpad
    // input thus zoom and orbit both end up with dual_axis data and axis data,
    // with the trackpad hack above, we know this is a trackpad, so let's get rid of
    // the axis data that would have come from the mouse - just for cleanliness
    // and as a reminder to get rid of this shite in the future
    if trackpad {
        if let Some(axis_data) = action_state.axis_data_mut(&CameraControl::Zoom) {
            // println!("eliding axis data in zoom {:?}", axis_data);
            axis_data.value = 0.0;
            axis_data.update_value = 0.0;
            axis_data.fixed_update_value = 0.0;
        }
        return None;
    }

    //use the `action_value` method to extract the total net amount that the mouse
    // wheel has travelled
    let zoom_delta = action_state.value(&CameraControl::Zoom);

    if zoom_delta == 0.0 {
        return None;
    }
    Some(zoom_delta)
}

fn pan_camera(
    mut query: Query<(&mut Transform, &ActionState<CameraControl>), With<PrimaryCamera>>,
    keycode: Res<ButtonInput<KeyCode>>,
    orientation: Res<CameraOrientation>,
) {
    if let Ok((mut camera_transform, action_state)) = query.get_single_mut() {
        let pan_vector = match should_pan(keycode, action_state) {
            Some(value) => value,
            None => return,
        };

        // To achieve consistent panning behavior regardless of the camera’s rotation,
        // we need to ensure that the panning movement is relative to the camera’s
        // current orientation.
        let right = camera_transform.rotation * orientation.config.axis_orbis;
        let up = camera_transform.rotation * orientation.config.axis_mundi;

        camera_transform.translation += right * -pan_vector.x;
        camera_transform.translation += up * pan_vector.y;
    }
}

// this code allows us to pan with mouse button pressed + ShiftLeft, just like
// Blender the following is a workaround for the fact that the ButtonlikeChord
// of MouseButton::Middle and KeyCode::ShiftLeft doesn't actually work
// but if ShiftLeft _is_ on then &CameraMovement::Orbit  will have the axis_pair
// needed for panning and we _didn't_ consume it in orbit if ShiftLeft was
// pressed hacky, hacky - but if LeafWing ever gets more sophisticated,
// ShiftLeft as a sentinel, and the following can go away and we can just get it
// from &CameraMovement::Pan
fn should_pan(keycode: Res<ButtonInput<KeyCode>>, action_state: &ActionState<CameraControl>) -> Option<Vec2> {
    let pan_vector = if keycode.pressed(KeyCode::ShiftLeft) {
        action_state.axis_pair(&CameraControl::Orbit)
    } else {
        action_state.axis_pair(&CameraControl::Pan)
    };

    if pan_vector == Vec2::ZERO {
        return None;
    }
    Some(pan_vector)
}

// i couldn't get this to work without hitting gimbal lock when consulting with
// chatGPT 4.o claude Sonnet 3.5 got it right on the first try - holy shit!
fn orbit_camera(
    mut q_camera: Query<(&mut Transform, &mut ActionState<CameraControl>), With<PrimaryCamera>>,
    camera_config: Res<CameraConfig>,
    keycode: Res<ButtonInput<KeyCode>>,
    orientation: Res<CameraOrientation>,
) {
    if let Ok((mut camera_transform, mut action_state)) = q_camera.get_single_mut() {
        let orbit_vector = match should_orbit(&mut action_state, keycode) {
            Some(value) => value,
            None => return,
        };

        let rotation_speed = camera_config.orbit_speed; //0.005;
                                                        // Assuming the target is at the origin - this may change in the future
                                                        // as the target could be the ship when we move into flying behind the ship
        let target = orientation.config.nexus;

        // this will change if we change our up vector to Z for FPSpaceship mode
        let up = orientation.config.axis_mundi.normalize();
        let right = camera_transform.right().normalize();

        // Create rotation quaternions
        let pitch_rotation = Quat::from_axis_angle(right, -orbit_vector.y * rotation_speed);
        let yaw_rotation = Quat::from_axis_angle(up, -orbit_vector.x * rotation_speed);

        // Combine rotations
        let rotation = yaw_rotation * pitch_rotation;

        // Apply rotation to the camera's position relative to the target
        let relative_position = camera_transform.translation - target;
        let new_relative_position = rotation * relative_position;

        // Update the camera's position and orientation
        camera_transform.translation = target + new_relative_position;
        camera_transform.rotation = rotation * camera_transform.rotation;

        elide_dual_axis_data(&mut action_state);
    }
}

// we're using a sentinel of ShiftLeft because we want the combination of
// ShiftLeft, MouseWheelMiddle to allow the mouse to pan. however orbit ends up
// with that data in the orbit_vector right now so we have to treat it as a
// sentinel if Pan has any data, orbit will also - but Pan will be the victor so
// we need to let that through as Pan is sequenced after this
fn should_orbit(
    camera_input: &mut Mut<ActionState<CameraControl>>,
    keycode: Res<ButtonInput<KeyCode>>,
) -> Option<Vec2> {
    let orbit_vector = camera_input.axis_pair(&CameraControl::Orbit);
    let pan_vector = camera_input.axis_pair(&CameraControl::Pan);

    if orbit_vector == Vec2::ZERO || pan_vector != Vec2::ZERO || keycode.pressed(KeyCode::ShiftLeft) {
        return None;
    }
    Some(orbit_vector)
}

// todo: #bevy_question - is there another way?
// clear out dual_axis data because orbit goes first and it always contains an
// orbit data value i can't figure out how to consume it
// this is necessary because otherwise the dual_axis_data shows up on zoom and
// on pan and i can't get the Chords I want without this - a fix would be if
// support for distinguishing between a touch pad scroll and a mouse scroll was
// added
fn elide_dual_axis_data(action_state: &mut Mut<ActionState<CameraControl>>) {
    // so we definitely are using the mouse wheel so get rid of any dual_axis shite
    if let Some(dual_axis_data) = action_state.dual_axis_data_mut(&CameraControl::Orbit) {
        //     println!("eliding orbit data in zoom {:?}", dual_axis_data);

        dual_axis_data.pair = Vec2::ZERO;
        dual_axis_data.update_pair = Vec2::ZERO;
        dual_axis_data.fixed_update_pair = Vec2::ZERO;
    }
}
