use crate::{
    boundary::Boundary,
    camera::{
        CameraOrder,
        RenderLayer,
        StarsCamera,
    },
    config::AppearanceConfig,
    input::CameraMovement,
    orientation::CameraOrientation,
    schedule::InGameSet,
};
use bevy::{
    core_pipeline::tonemapping::Tonemapping,
    input::{
        gestures::PinchGesture,
        mouse::{
            MouseScrollUnit,
            MouseWheel,
        },
    },
    prelude::{
        KeyCode::ShiftLeft,
        *,
    },
    render::view::RenderLayers,
};
use leafwing_input_manager::prelude::*;

pub struct PrimaryCameraPlugin;

impl Plugin for PrimaryCameraPlugin {
    fn build(&self, app: &mut App) {
        let appearance = AppearanceConfig::default();

        app.insert_resource(ClearColor(
            appearance
                .clear_color
                .darker(appearance.clear_color_darkening_factor),
        ))
        .add_systems(Startup, spawn_primary_camera)
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
                .chain(), // .in_set(InGameSet::UserInput),
        )
        .add_systems(Update, update_clear_color.in_set(InGameSet::EntityUpdates));
    }
}

// this allows us to use Inspector reflection to manually update ClearColor to
// different values while the game is running from the ui_for_resources provided
// by bevy_inspector_egui
fn update_clear_color(app_clear_color: Res<AppearanceConfig>, mut clear_color: ResMut<ClearColor>) {
    clear_color.0 = app_clear_color
        .clear_color
        .darker(app_clear_color.clear_color_darkening_factor);
}

#[derive(Component, Debug)]
pub struct PrimaryCamera;

fn home_camera(
    orientation: Res<CameraOrientation>,
    mut camera_transform: Query<
        (&mut Transform, &ActionState<CameraMovement>),
        With<PrimaryCamera>,
    >,
) {
    if let Ok((mut transform, action_state)) = camera_transform.get_single_mut() {
        if action_state.just_pressed(&CameraMovement::Home) {
            *transform = orientation.config.locus;
        }
    }
}

pub fn spawn_primary_camera(
    boundary: Res<Boundary>,
    mut commands: Commands,
    mut orientation: ResMut<CameraOrientation>,
    mut q_stars_camera: Query<Entity, With<StarsCamera>>,
) {
    let clear_color = Color::srgba(0., 0., 0., 0.);

    // we know we have one because we spawn the stars camera prior to this system
    // we're going to attach it to the primary as a child so it always has the same
    // view as the primary camera but can show the stars with bloom while the
    // primary shows everything else
    let stars_camera_entity = q_stars_camera
        .get_single_mut()
        .expect("why in god's name is there no star's camera?");

    let primary_camera = Camera3dBundle {
        camera: Camera {
            hdr: true,
            order: CameraOrder::Game.order(),
            clear_color: ClearColorConfig::Custom(clear_color),
            ..default()
        },
        tonemapping: Tonemapping::TonyMcMapface,
        transform: Transform::from_xyz(0.0, 0.0, boundary.transform.scale.z * 2.)
            .looking_at(orientation.config.nexus, orientation.config.axis_mundi),

        ..default()
    };

    orientation.config.locus = primary_camera.transform;

    commands
        .spawn(primary_camera)
        .insert(RenderLayers::from_layers(RenderLayer::Game.layers()))
        .insert(InputManagerBundle::with_map(
            CameraMovement::camera_input_map(),
        ))
        .add_child(stars_camera_entity)
        .insert(PrimaryCamera);
}

fn pinch_to_zoom(
    mut query: Query<&mut Transform, With<PrimaryCamera>>,
    mut pinch_gesture_events: EventReader<PinchGesture>,
    config: Res<AppearanceConfig>,
) {
    for event in pinch_gesture_events.read() {
        if let Ok(mut transform) = query.get_single_mut() {
            impl_zoom(config.zoom_sensitivity_pinch, &mut transform, event.0);
        }
    }
}

fn zoom_camera(
    mut query: Query<(&mut Transform, &mut ActionState<CameraMovement>), With<PrimaryCamera>>,
    mut mouse_wheel_events: EventReader<MouseWheel>,
    config: Res<AppearanceConfig>,
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
    action_state: &mut Mut<ActionState<CameraMovement>>,
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
        if let Some(axis_data) = action_state.axis_data_mut(&CameraMovement::Zoom) {
            // println!("eliding axis data in zoom {:?}", axis_data);
            axis_data.value = 0.0;
            axis_data.update_value = 0.0;
            axis_data.fixed_update_value = 0.0;
        }
        return None;
    }

    //use the `action_value` method to extract the total net amount that the mouse
    // wheel has travelled
    let zoom_delta = action_state.value(&CameraMovement::Zoom);

    if zoom_delta == 0.0 {
        return None;
    }
    Some(zoom_delta)
}

fn pan_camera(
    mut query: Query<(&mut Transform, &ActionState<CameraMovement>), With<PrimaryCamera>>,
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
fn should_pan(
    keycode: Res<ButtonInput<KeyCode>>,
    action_state: &ActionState<CameraMovement>,
) -> Option<Vec2> {
    let pan_vector = if keycode.pressed(ShiftLeft) {
        action_state.axis_pair(&CameraMovement::Orbit)
    } else {
        action_state.axis_pair(&CameraMovement::Pan)
    };

    if pan_vector == Vec2::ZERO {
        return None;
    }
    Some(pan_vector)
}

// i couldn't get this to work without hitting gimbal lock when consulting with
// chatGPT 4.o claude Sonnet 3.5 got it right on the first try - holy shit!
fn orbit_camera(
    mut query: Query<(&mut Transform, &mut ActionState<CameraMovement>), With<PrimaryCamera>>,
    keycode: Res<ButtonInput<KeyCode>>,
    orientation: Res<CameraOrientation>,
) {
    if let Ok((mut camera_transform, mut action_state)) = query.get_single_mut() {
        let orbit_vector = match should_orbit(keycode, &mut action_state) {
            Some(value) => value,
            None => return,
        };

        let rotation_speed = 0.005;
        // Assuming the target is at the origin - this may change in the future
        // as the target could be the ship when we move into flying behind the ship
        let target = orientation.config.nexus;

        // this will change if we change our up vector to Z for FPSpaceship mode
        let up = orientation.config.axis_mundi;
        let right = camera_transform.right().as_vec3();

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
    keycode: Res<ButtonInput<KeyCode>>,
    action_state: &mut Mut<ActionState<CameraMovement>>,
) -> Option<Vec2> {
    let orbit_vector = action_state.axis_pair(&CameraMovement::Orbit);
    let pan_vector = action_state.axis_pair(&CameraMovement::Pan);

    if orbit_vector == Vec2::ZERO || pan_vector != Vec2::ZERO || keycode.pressed(ShiftLeft) {
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
fn elide_dual_axis_data(action_state: &mut Mut<ActionState<CameraMovement>>) {
    // so we definitely are using the mouse wheel so get rid of any dual_axis shite
    if let Some(dual_axis_data) = action_state.dual_axis_data_mut(&CameraMovement::Orbit) {
        //     println!("eliding orbit data in zoom {:?}", dual_axis_data);

        dual_axis_data.pair = Vec2::ZERO;
        dual_axis_data.update_pair = Vec2::ZERO;
        dual_axis_data.fixed_update_pair = Vec2::ZERO;
    }
}
