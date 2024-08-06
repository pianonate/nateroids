use crate::game_scale::GameScale;
use crate::{boundary::Boundary, camera::spawn_camera};
use bevy::{
    core_pipeline::{
        bloom::{BloomCompositeMode, BloomSettings},
        tonemapping::Tonemapping,
    },
    prelude::*,
    render::view::RenderLayers,
};
use rand::Rng;

const STARS_CAMERA_ORDER: isize = 0;
const STARS_LAYER: usize = 1;
pub const GAME_CAMERA_ORDER: isize = 1;
pub const GAME_LAYER: usize = 0;

const BATCH_SIZE: usize = 100;

pub struct StarsPlugin;

impl Plugin for StarsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_camera.before(spawn_camera))
            .add_systems(Update, update_stars)
            // .add_systems(Startup, setup_stars);
            .insert_resource(StarSpawnTimer(Timer::from_seconds(
                0.05,
                TimerMode::Repeating,
            )))
            .insert_resource(StarCounter(0))
            .add_systems(Update, spawn_star_tasks);
    }
}

fn setup_camera(mut commands: Commands) {
    let camera3d = Camera3dBundle {
        camera: Camera {
            order: STARS_CAMERA_ORDER,
            hdr: true, // 1. HDR is required for bloom
            ..default()
        },
        tonemapping: Tonemapping::BlenderFilmic,
        ..default()
    };

    commands
        .spawn(camera3d)
        .insert(RenderLayers::layer(STARS_LAYER))
        .insert(BloomSettings::NATURAL)
        .insert(StarsCamera);
}

#[derive(Component)]
pub struct StarsCamera;

fn update_stars(
    mut commands: Commands,
    mut camera: Query<(Entity, Option<&mut BloomSettings>), With<StarsCamera>>,
    keycode: Res<ButtonInput<KeyCode>>,
) {
    let bloom_settings = camera.single_mut();

    match bloom_settings {
        (entity, Some(mut bloom_settings)) => {
            bloom_settings.intensity = 0.9;
            bloom_settings.low_frequency_boost = 0.6;
            bloom_settings.low_frequency_boost_curvature = 0.5;
            bloom_settings.high_pass_frequency = 0.5;
            bloom_settings.composite_mode = BloomCompositeMode::EnergyConserving;

            if keycode.just_pressed(KeyCode::KeyB) {
                println!("bloom off");
                commands.entity(entity).remove::<BloomSettings>();
            }
        }
        (entity, None) => {
            if keycode.just_pressed(KeyCode::KeyB) {
                println!("bloom on");
                commands.entity(entity).insert(BloomSettings::NATURAL);
            }
        }
    }
}

#[derive(Resource)]
struct StarSpawnTimer(Timer);

#[derive(Resource)]
struct StarCounter(usize);

// generate BATCH_SIZE stars at a time until you get to the desired number of stars
// this will fill them in as the program starts and looks cooler that way
#[allow(clippy::too_many_arguments)]
fn spawn_star_tasks(
    mut commands: Commands,
    game_scale: Res<GameScale>,
    boundary: Res<Boundary>,
    time: Res<Time>,
    mut timer: ResMut<StarSpawnTimer>,
    mut counter: ResMut<StarCounter>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if timer.0.tick(time.delta()).just_finished() && counter.0 < game_scale.star_count {
        let longest_diagonal = boundary.longest_diagonal;
        let inner_sphere_radius = longest_diagonal + game_scale.star_field_inner_diameter;
        let outer_sphere_radius = inner_sphere_radius + game_scale.star_field_outer_diameter;

        let game_scale = game_scale.clone(); // Clone the game_scale resource

        let star_mesh_handle = meshes.add(Sphere::new(0.5).mesh().ico(5).unwrap());

        let stars_to_spawn = (game_scale.star_count - counter.0).min(BATCH_SIZE);

        for _ in 0..stars_to_spawn {
            let mut rng = rand::thread_rng();
            let point = {
                let u: f32 = rng.gen_range(0.0..1.0);
                let v: f32 = rng.gen_range(0.0..1.0);
                let theta = u * std::f32::consts::PI * 2.0;
                let phi = (2.0 * v - 1.0).acos();
                let r = rng.gen_range(inner_sphere_radius..outer_sphere_radius);

                let x = r * theta.cos() * phi.sin();
                let y = r * theta.sin() * phi.sin();
                let z = r * phi.cos();

                Vec3::new(x, y, z)
            };

            // Increase the likelihood of generating higher values for R, G, B
            let emissive_r = rng.gen_range(8.0..15.0);
            let emissive_g = rng.gen_range(8.0..15.0);
            let emissive_b = rng.gen_range(8.0..15.0);

            let transform =
                Transform::from_translation(point).with_scale(Vec3::splat(game_scale.star_scale));

            let material = materials.add(StandardMaterial {
                emissive: LinearRgba::rgb(emissive_r, emissive_g, emissive_b),
                ..default()
            });

            commands
                .spawn(PbrBundle {
                    mesh: star_mesh_handle.clone(),
                    material,
                    transform,
                    ..default()
                })
                .insert(RenderLayers::layer(STARS_LAYER));
        }

        counter.0 += stars_to_spawn;
    }
}
