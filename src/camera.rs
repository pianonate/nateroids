use crate::stars::{StarsCamera, GAME_CAMERA_ORDER, GAME_LAYER};
use crate::{boundary::Boundary, input::CameraMovement, schedule::InGameSet};
use bevy::{
    color::palettes::css,
    //core_pipeline::Skybox,
    input::mouse::{MouseScrollUnit, MouseWheel},
    prelude::{Color::Srgba, KeyCode::ShiftLeft, *},
    render::view::RenderLayers,
};
use leafwing_input_manager::prelude::*;

const DEFAULT_CLEAR_COLOR_DARKENING_FACTOR: f32 = 0.019;
const DEFAULT_CLEAR_COLOR: Color = Srgba(css::MIDNIGHT_BLUE);
const DEFAULT_AMBIENT_LIGHT_BRIGHTNESS: f32 = 1_000.;

#[derive(Resource, Reflect, Debug, Default)]
#[reflect(Resource)]
pub struct Appearance {
    color: Color,
    darkening_factor: f32,
    ambient_light_brightness: f32,
}

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Appearance {
            color: DEFAULT_CLEAR_COLOR,
            darkening_factor: DEFAULT_CLEAR_COLOR_DARKENING_FACTOR,
            ambient_light_brightness: DEFAULT_AMBIENT_LIGHT_BRIGHTNESS,
        })
        .insert_resource(ClearColor(
            DEFAULT_CLEAR_COLOR.darker(DEFAULT_CLEAR_COLOR_DARKENING_FACTOR),
        ))
        .insert_resource(AmbientLight {
            color: default(),
            brightness: 0.2,
        })
        .add_systems(Startup, spawn_camera)
        .add_systems(
            Update,
            (
                // order matters because we hack around the input manager
                // that doesn't yet support trackpads
                zoom_camera,
                orbit_camera,
                pan_camera,
            )
                .chain()
                .in_set(InGameSet::UserInput),
        )
        .add_systems(Update, update_clear_color.in_set(InGameSet::EntityUpdates));
    }
}

// this allows us to use Inspector reflection to manually update ClearColor to different values
// while the game is running from the ui_for_resources provided by bevy_inspector_egui
fn update_clear_color(
    app_clear_color: Res<Appearance>,
    mut clear_color: ResMut<ClearColor>,
    mut ambient_light: ResMut<AmbientLight>,
) {
    clear_color.0 = app_clear_color
        .color
        .darker(app_clear_color.darkening_factor);

    ambient_light.brightness = app_clear_color.ambient_light_brightness;
}

#[derive(Component, Debug)]
pub struct PrimaryCamera;

pub fn spawn_camera(
    mut commands: Commands,
    boundary: Res<Boundary>,
    mut q_stars_camera: Query<(Entity, &mut Transform), With<StarsCamera>>,
) {
    let clear_color = Color::srgba(0., 0., 0., 0.);

    // we know we have one because we spawn the stars camera prior to this system
    // we're going to change its transform to zero and attach it to this as a child
    // so it always goes wherever we go
    let (stars_camera_entity, mut stars_camera_transform) =
        q_stars_camera.get_single_mut().unwrap();
    stars_camera_transform.translation = Vec3::ZERO;

    commands
        .spawn((
            Camera3dBundle {
                camera: Camera {
                    order: GAME_CAMERA_ORDER,
                    clear_color: ClearColorConfig::Custom(clear_color),
                    ..default()
                },
                transform: Transform::from_xyz(0.0, 0.0, boundary.transform.scale.z * 2.)
                    .looking_at(Vec3::ZERO, Vec3::Y),

                ..default()
            },
            // if you want to add a skybox on a level, you can do it here
            // Skybox {
            //     image: scene_assets.cubemap.image_handle.clone(),
            //     brightness: 1000.0,
            // },
        ))
        .insert(RenderLayers::layer(GAME_LAYER))
        .insert(InputManagerBundle::with_map(
            CameraMovement::camera_input_map(),
        ))
        .add_child(stars_camera_entity)
        .insert(PrimaryCamera);
}

fn zoom_camera(
    mut query: Query<(&mut Transform, &mut ActionState<CameraMovement>), With<PrimaryCamera>>,
    mut mouse_wheel_events: EventReader<MouseWheel>,
) {
    let mut trackpad = false;

    // hack to determine if the input was from a mouse or a trackpad
    for event in mouse_wheel_events.read() {
        trackpad = match event.unit {
            MouseScrollUnit::Line => false,
            MouseScrollUnit::Pixel => true,
        };
    }

    if let Ok((mut transform, mut action_state)) = query.get_single_mut() {
        // leafwing doesn't yet allow us to distinguish between mouse input and trackpad input
        // zo zoom and orbit both end up with dual_axis data and axis data,
        // with the trackpad hack above, we know this is a trackpad, so let's get rid of the
        // axis data that would have come from the mouse - just for cleanliness and as a reminder
        // to get rid of this shite in the future
        if trackpad {
            if let Some(axis_data) = action_state.axis_data_mut(&CameraMovement::Zoom) {
                // println!("eliding axis data in zoom {:?}", axis_data);
                axis_data.value = 0.0;
                axis_data.update_value = 0.0;
                axis_data.fixed_update_value = 0.0;
            }
            return;
        }

        //use the `action_value` method to extract the total net amount that the mouse wheel has travelled
        let zoom_delta = action_state.value(&CameraMovement::Zoom);

        if zoom_delta == 0.0 {
            return;
        }

        let zoom_update = 1. - zoom_delta;

        transform.translation.z += zoom_update;

        println!(
            "zoom_delta {} translation {}",
            zoom_delta, transform.translation.z
        );

        elide_dual_axis_data(&mut action_state);
    }
}

// To achieve consistent panning behavior regardless of the camera’s rotation, we need to ensure
// that the panning movement is relative to the camera’s current orientation. In Blender,
// panning always moves the view in the screen space direction, which means it accounts
// for the camera’s rotation. let's do the same here - it's easier to parse
fn pan_camera(
    mut query: Query<(&mut Transform, &ActionState<CameraMovement>), With<PrimaryCamera>>,
    keycode: Res<ButtonInput<KeyCode>>,
) {
    if let Ok((mut camera_transform, action_state)) = query.get_single_mut() {
        // work around for the fact that the ButtonlikeChord of
        // MouseButton::Middle and KeyCode::ShiftLeft don't really work
        // but if ShiftLeft is on then Orbit will have the axis_pair
        // and we didn't consume it in orbit if ShiftLeft was turned on
        // hacky, hacky - but if LeafWing ever gets more sophisticated then this can go away
        let pan_vector = if keycode.pressed(ShiftLeft) {
            action_state.axis_pair(&CameraMovement::Orbit)
        } else {
            action_state.axis_pair(&CameraMovement::Pan)
        };

        if pan_vector == Vec2::ZERO {
            return;
        }

        let right = camera_transform.rotation * Vec3::X;
        let up = camera_transform.rotation * Vec3::Y;

        camera_transform.translation += right * -pan_vector.x;
        camera_transform.translation += up * pan_vector.y;
    }
}

// i couldn't get this to work without hitting gimbal lock when consulting with chatGPT 4.o
// claude Sonnet 3.5 got it right on the first try - holy shit!
fn orbit_camera(
    mut query: Query<(&mut Transform, &mut ActionState<CameraMovement>), With<PrimaryCamera>>,
    keycode: Res<ButtonInput<KeyCode>>,
) {
    if let Ok((mut camera_transform, mut action_state)) = query.get_single_mut() {
        let orbit_vector = action_state.axis_pair(&CameraMovement::Orbit);
        let pan_vector = action_state.axis_pair(&CameraMovement::Pan);

        if orbit_vector == Vec2::ZERO
            || pan_vector != Vec2::ZERO
            || keycode.pressed(KeyCode::ShiftLeft)
        {
            return;
        }

        let rotation_speed = 0.005;
        // Assuming the target is at the origin - this may change in the future
        // as the target could be the ship when we move into flying behind the ship
        let target = Vec3::ZERO;

        // this will change if we change our up vector to Z for FPSpaceship mode
        let up = Vec3::Y;
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

// todo: #bevyquestion - is there another way?
// clear out dual_axis data because orbit goes first and it always contains an orbit data value
// i can't figure out how to consume it
// this is necessary because otherwise the dual_axis_data shows up on zoom and on pan
// and i can't get the Chords I want without this - a fix would be if
// support for distinguishing between a touch pad scroll and a mouse scroll was added
fn elide_dual_axis_data(action_state: &mut Mut<ActionState<CameraMovement>>) {
    // so we definitely are using the mouse wheel so get rid of any dual_axis shite
    if let Some(dual_axis_data) = action_state.dual_axis_data_mut(&CameraMovement::Orbit) {
        //     println!("eliding orbit data in zoom {:?}", dual_axis_data);

        dual_axis_data.pair = Vec2::ZERO;
        dual_axis_data.update_pair = Vec2::ZERO;
        dual_axis_data.fixed_update_pair = Vec2::ZERO;
    }
}
