use crate::{
    camera::camera_config::LightConfig,
    orientation::CameraOrientation,
};
use bevy::prelude::*;

pub struct DirectionalLightsPlugin;

impl Plugin for DirectionalLightsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AmbientLight>()
            .add_systems(Update, manage_lighting);
    }
}

#[derive(Resource, Debug, PartialEq, Eq, Clone, Copy)]
pub enum LightPosition {
    Front,
    Back,
    Top,
    Bottom,
    Left,
    Right,
}

impl LightPosition {
    pub fn get_rotation(&self, orientation: &CameraOrientation) -> RotationInfo {
        use std::f32::consts::{
            FRAC_PI_2,
            PI,
        };
        match self {
            LightPosition::Right => RotationInfo {
                axis:  orientation.config.axis_mundi,
                angle: FRAC_PI_2,
            },
            LightPosition::Left => RotationInfo {
                axis:  orientation.config.axis_mundi,
                angle: -FRAC_PI_2,
            },
            LightPosition::Front => RotationInfo {
                axis:  orientation.config.axis_orbis,
                angle: 0.,
            },
            LightPosition::Back => RotationInfo {
                axis:  orientation.config.axis_orbis,
                angle: PI,
            },
            LightPosition::Bottom => RotationInfo {
                axis:  orientation.config.axis_orbis,
                angle: FRAC_PI_2,
            },
            LightPosition::Top => RotationInfo {
                axis:  orientation.config.axis_orbis,
                angle: -FRAC_PI_2,
            },
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct RotationInfo {
    pub axis:  Vec3,
    pub angle: f32,
}

// looked this up on github - so it doesn't really matter where it's placed...
//
// Directional light sources are modelled to be at infinity and have parallel
// rays. As such they do not have a position in practical terms and only the
// rotation matters. The direction of the light is defined by the forward
// direction and by default the -z axis is forwards (right-handed, y-up, z
// points backwards and -z is forwards). Rotations are then applied to a Vec3 of
// (0,0,-1) to calculate the transform’s forward direction.

#[derive(Component)]
struct LightDirection(LightPosition);

fn manage_lighting(
    mut commands: Commands,
    mut ambient_light: ResMut<AmbientLight>,
    light_config: Res<LightConfig>,
    camera_orientation: Res<CameraOrientation>,
    mut query: Query<(Entity, &mut DirectionalLight, &LightDirection)>,
) {
    if !light_config.is_changed() {
        return;
    }

    ambient_light.brightness = light_config.ambient_light_brightness;
    ambient_light.color = light_config.ambient_light_color;

    // iterate through all possible positions to see if any of them exist...
    // if it's been enabled and it doesn't exist then spawn it
    // if it has changed then update it to what it's changed to
    for position in [
        LightPosition::Right,
        LightPosition::Left,
        LightPosition::Front,
        LightPosition::Back,
        LightPosition::Bottom,
        LightPosition::Top,
    ]
    .iter()
    {
        let settings = light_config.get_light_settings(*position);

        // we always spawn a light with its current LightDirection - see
        // if we have the current loop's position in an already spawned entity
        let existing_light = query.iter_mut().find(|(_, _, dir)| dir.0 == *position);

        let light_rotation = position.get_rotation(&camera_orientation);

        match (existing_light, settings.enabled) {
            (Some((_, mut light, _)), true) => {
                // Update existing light
                light.color = settings.color;
                light.illuminance = settings.illuminance;
                light.shadows_enabled = settings.shadows_enabled;
            },
            (Some((entity, _, _)), false) => {
                // Remove disabled light
                commands.entity(entity).despawn();
            },
            (None, true) => {
                // Spawn new light
                commands.spawn((
                    DirectionalLightBundle {
                        directional_light: DirectionalLight {
                            color: settings.color,
                            illuminance: settings.illuminance,
                            shadows_enabled: settings.shadows_enabled,
                            ..default()
                        },
                        transform: Transform::from_rotation(Quat::from_axis_angle(
                            light_rotation.axis,
                            light_rotation.angle,
                        )),
                        ..default()
                    },
                    LightDirection(*position),
                ));
            },
            (None, false) => {}, // Do nothing for disabled lights that don't exist
        }
    }
}
