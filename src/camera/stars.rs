use crate::{
    boundary::Boundary,
    camera::primary_camera::spawn_primary_camera,
    config::AppearanceConfig,
    input::GlobalAction,
};
use bevy::{
    core_pipeline::{
        bloom::BloomSettings,
        tonemapping::Tonemapping,
    },
    prelude::*,
    render::view::RenderLayers,
};
use std::ops::Range;

use crate::camera::{
    CameraOrder,
    RenderLayer,
};
use leafwing_input_manager::action_state::ActionState;
use rand::{
    prelude::ThreadRng,
    Rng,
};

pub struct StarsPlugin;

impl Plugin for StarsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<StarBloom>()
            .add_systems(Startup, spawn_star_camera.before(spawn_primary_camera))
            .add_systems(Startup, (spawn_stars, setup_star_rendering).chain())
            .add_systems(Update, (toggle_stars, update_bloom_settings));
    }
}

#[derive(Debug, Clone, Reflect, Resource)]
#[reflect(Resource)]
pub struct StarConfig {
    pub batch_size_replace:            usize,
    pub duration_replace_timer:        f32,
    pub star_color:                    Range<f32>,
    pub star_color_white_probability:  f32,
    pub star_color_white_start_ratio:  f32,
    pub star_count:                    usize,
    pub star_radius_max:               f32,
    pub star_radius_min:               f32,
    pub star_field_inner_diameter:     f32,
    pub star_field_outer_diameter:     f32,
    pub start_twinkling_delay:         f32,
    pub twinkle_duration:              Range<f32>,
    pub twinkle_intensity:             Range<f32>,
    pub twinkle_choose_multiple_count: usize,
}

impl Default for StarConfig {
    fn default() -> Self {
        Self {
            batch_size_replace:            10,
            duration_replace_timer:        1.,
            star_count:                    500,
            star_color:                    -30.0..30.0,
            star_color_white_probability:  0.85,
            star_color_white_start_ratio:  0.7,
            star_radius_max:               2.5,
            star_radius_min:               0.3,
            star_field_inner_diameter:     200.,
            star_field_outer_diameter:     400.,
            start_twinkling_delay:         0.5,
            twinkle_duration:              0.5..2.,
            twinkle_intensity:             10.0..20.,
            twinkle_choose_multiple_count: 2, // stars to look at each update
        }
    }
}

#[derive(Resource, Clone)]
struct StarBloom {
    settings: BloomSettings,
}

impl Default for StarBloom {
    fn default() -> Self {
        let config = AppearanceConfig::default();
        let mut bloom_settings = BloomSettings::NATURAL;
        bloom_settings.intensity = config.bloom_intensity;
        bloom_settings.low_frequency_boost = config.bloom_low_frequency_boost;
        bloom_settings.high_pass_frequency = config.bloom_high_pass_frequency;
        Self {
            settings: bloom_settings,
        }
    }
}

// allows Appearance settings to propagate back to StarBloom.settings so
// that on changes we can apply a clone of those settings back to
// the camera
impl StarBloom {
    fn update_from_config(&mut self, config: &AppearanceConfig) {
        self.settings.intensity = config.bloom_intensity;
        self.settings.low_frequency_boost = config.bloom_low_frequency_boost;
        self.settings.high_pass_frequency = config.bloom_high_pass_frequency;
    }
}

// if Appearance changes (ignore the fact that anything can change - that's
// fine) then propagate bloom settings back to the resource, and then clone it
// back onto the camera
fn update_bloom_settings(
    mut star_bloom: ResMut<StarBloom>,
    appearance_config: Res<AppearanceConfig>,
    mut query: Query<&mut BloomSettings, With<StarsCamera>>,
) {
    if appearance_config.is_changed() {
        star_bloom.update_from_config(&appearance_config);
        for mut bloom_settings in query.iter_mut() {
            *bloom_settings = star_bloom.settings.clone();
        }
    }
}

// star camera uses bloom so it needs to be in its own layer as we don't
// want that effect on the colliders
fn spawn_star_camera(mut commands: Commands, star_bloom: Res<StarBloom>) {
    let camera3d = Camera3dBundle {
        camera: Camera {
            order: CameraOrder::Stars.order(),
            hdr: true, // 1. HDR is required for bloom
            ..default()
        },
        tonemapping: Tonemapping::BlenderFilmic,
        ..default()
    };

    commands
        .spawn(camera3d)
        .insert(RenderLayers::from_layers(RenderLayer::Stars.layers()))
        .insert(star_bloom.settings.clone())
        .insert(StarsCamera);
}

#[derive(Component)]
pub struct StarsCamera;

// remove and insert BloomSettings to toggle them off and on
// this can probably be removed now that bloom is pretty well working...
fn toggle_stars(
    mut commands: Commands,
    mut camera: Query<(Entity, Option<&mut BloomSettings>), With<StarsCamera>>,
    user_input: Res<ActionState<GlobalAction>>,
    star_bloom: Res<StarBloom>,
) {
    let current_bloom_settings = camera.single_mut();

    match current_bloom_settings {
        (entity, Some(_)) => {
            if user_input.just_pressed(&GlobalAction::Stars) {
                println!("stars off");
                commands.entity(entity).remove::<BloomSettings>();
            }
        },
        (entity, None) => {
            if user_input.just_pressed(&GlobalAction::Stars) {
                println!("stars on");
                commands.entity(entity).insert(star_bloom.settings.clone());
            }
        },
    }
}

#[derive(Component, Default)]
pub struct Star {
    position:     Vec3,
    radius:       f32,
    pub emissive: Vec4,
}

// just set up the entities with their positions - we'll add an emissive
// StandardMaterial separately
fn spawn_stars(mut commands: Commands, config: Res<StarConfig>, boundary: Res<Boundary>) {
    let longest_diagonal = boundary.longest_diagonal;
    let inner_sphere_radius = longest_diagonal + config.star_field_inner_diameter;
    let outer_sphere_radius = inner_sphere_radius + config.star_field_outer_diameter;

    let mut rng = rand::thread_rng();

    for _ in 0..config.star_count {
        let point = get_star_position(inner_sphere_radius, outer_sphere_radius, &mut rng);
        let radius = rng.gen_range(config.star_radius_min..config.star_radius_max);
        let emissive = get_star_color(&config, &mut rng);

        commands.spawn((
            Star {
                position: point,
                radius,
                emissive,
            },
            RenderLayers::from_layers(RenderLayer::Stars.layers()),
        ));
    }
}

fn get_star_position(
    inner_sphere_radius: f32,
    outer_sphere_radius: f32,
    rng: &mut ThreadRng,
) -> Vec3 {
    let u: f32 = rng.gen_range(0.0..1.0);
    let v: f32 = rng.gen_range(0.0..1.0);
    let theta = u * std::f32::consts::PI * 2.0;
    let phi = (2.0 * v - 1.0).acos();
    let r = rng.gen_range(inner_sphere_radius..outer_sphere_radius);

    let x = r * theta.cos() * phi.sin();
    let y = r * theta.sin() * phi.sin();
    let z = r * phi.cos();

    Vec3::new(x, y, z)
}

// fn get_star_color(config: &StarConfig, rng: &mut impl Rng) -> Vec4 {
//     let end = config.star_color.end;
//     let color_start = config.star_color.start;
//     let white_start = end * config.star_color_white_start_ratio; // 80% of
// end value
//
//     let start = if rng.gen::<f32>() < config.star_color_white_probability {
//         white_start
//     } else {
//         color_start
//     };
//
//     Vec4::new(
//         rng.gen_range(start..end),
//         rng.gen_range(start..end),
//         rng.gen_range(start..end),
//         rng.gen_range(start..end),
//     )
// }

fn get_star_color(config: &StarConfig, rng: &mut impl Rng) -> Vec4 {
    let end = config.star_color.end;
    let color_start = config.star_color.start;
    let white_start = end * config.star_color_white_start_ratio;

    let start = if rng.gen::<f32>() < config.star_color_white_probability {
        white_start
    } else {
        color_start
    };

    // Generate initial color components
    let mut r = rng.gen_range(start..end);
    let mut g = rng.gen_range(start..end);
    let mut b = rng.gen_range(start..end);

    // Ensure minimum brightness
    let min_brightness = start + (end - start) * 0.2; // 20% above start
    let current_brightness = r.max(g).max(b);

    if current_brightness < min_brightness {
        let scale = min_brightness / current_brightness;
        r *= scale;
        g *= scale;
        b *= scale;
    }

    // Alpha can remain as is
    let a = rng.gen_range(start..end);

    Vec4::new(r, g, b, a)
}

// add the emissive standard material generated randomly in spawn_stars
fn setup_star_rendering(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    stars: Query<(Entity, &Star)>,
) {
    let mesh = meshes.add(Sphere::new(1.));

    for (entity, star) in stars.iter() {
        let material = materials.add(StandardMaterial {
            emissive: LinearRgba::new(
                star.emissive.x,
                star.emissive.y,
                star.emissive.z,
                star.emissive.w,
            ),
            ..default()
        });

        commands.entity(entity).insert(MaterialMeshBundle {
            mesh: mesh.clone(),
            material,
            transform: Transform::from_translation(star.position)
                .with_scale(Vec3::splat(star.radius)),
            ..default()
        });
    }
}
