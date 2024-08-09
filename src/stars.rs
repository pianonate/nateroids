use crate::{
    boundary::Boundary,
    camera::spawn_camera,
    config::{
        AppearanceConfig,
        CameraOrder,
        RenderLayer,
        StarConfig,
    },
    despawn::despawn,
    input::GlobalAction,
};
use bevy::{
    core_pipeline::{
        bloom::BloomSettings,
        tonemapping::Tonemapping,
    },
    ecs::system::SystemParam,
    prelude::*,
    render::view::RenderLayers,
};
use leafwing_input_manager::action_state::ActionState;
use rand::{
    prelude::SliceRandom,
    thread_rng,
    Rng,
};

pub struct StarsPlugin;

impl Plugin for StarsPlugin {
    fn build(&self, app: &mut App,) {
        let star_config = StarConfig::default();

        app.add_systems(Startup, setup_camera.before(spawn_camera,),)
            .add_systems(Update, toggle_stars,)
            .init_resource::<StarBloom>()
            .insert_resource(StarSpawnTimer(Timer::from_seconds(
                star_config.duration_spawn_timer,
                TimerMode::Repeating,
            ),),)
            .insert_resource(StarReplaceTimer(Timer::from_seconds(
                star_config.duration_replace_timer,
                TimerMode::Repeating,
            ),),)
            // this can run while paused or splashing for now
            .add_systems(Update, (spawn_star_tasks, replace_stars,),)
            .add_systems(Update, (rotate_sphere, update_bloom_settings,),);
    }
}

#[derive(Resource, Clone,)]
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
    fn update_from_config(&mut self, config: &AppearanceConfig,) {
        self.settings.intensity = config.bloom_intensity;
        self.settings.low_frequency_boost = config.bloom_low_frequency_boost;
        self.settings.high_pass_frequency = config.bloom_high_pass_frequency;
    }
}

// if Appearance changes (ignore the fact that anything can change - that's
// fine) then propagate bloom settings back to the resource, and then clone it
// back onto the camera
fn update_bloom_settings(
    mut star_bloom: ResMut<StarBloom,>,
    appearance_config: Res<AppearanceConfig,>,
    mut query: Query<&mut BloomSettings, With<StarsCamera,>,>,
) {
    if appearance_config.is_changed() {
        star_bloom.update_from_config(&appearance_config,);
        for mut bloom_settings in query.iter_mut() {
            *bloom_settings = star_bloom.settings.clone();
        }
    }
}

fn setup_camera(mut commands: Commands, star_bloom: Res<StarBloom,>,) {
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
        .spawn(camera3d,)
        .insert(RenderLayers::from_layers(RenderLayer::Stars.layers(),),)
        .insert(star_bloom.settings.clone(),)
        .insert(StarsCamera,);
}

#[derive(Component,)]
struct GameSphere;

// currently we're not displaying this - it's called but it never gets an Ok
// so it's a no-op
// keep this if you want to put the bounding sphere back in play...
fn rotate_sphere(mut query: Query<&mut Transform, With<GameSphere,>,>,) {
    if let Ok(mut transform,) = query.get_single_mut() {
        let delta_rotation = Quat::from_rotation_y(0.001,);

        transform.rotation *= delta_rotation;
    }
}

#[derive(Component,)]
pub struct StarsCamera;

fn toggle_stars(
    mut commands: Commands,
    mut camera: Query<(Entity, Option<&mut BloomSettings,>,), With<StarsCamera,>,>,
    user_input: Res<ActionState<GlobalAction,>,>,
    star_bloom: Res<StarBloom,>,
) {
    let current_bloom_settings = camera.single_mut();

    match current_bloom_settings {
        (entity, Some(_,),) => {
            if user_input.just_pressed(&GlobalAction::Stars,) {
                println!("stars off");
                commands.entity(entity,).remove::<BloomSettings>();
            }
        },
        (entity, None,) => {
            if user_input.just_pressed(&GlobalAction::Stars,) {
                println!("stars on");
                commands
                    .entity(entity,)
                    .insert(star_bloom.settings.clone(),);
            }
        },
    }
}

#[derive(Resource,)]
struct StarSpawnTimer(Timer,);

#[derive(Resource,)]
struct StarReplaceTimer(Timer,);

#[derive(Component,)]
pub struct Stars;

// just used to simplify the param list
#[derive(SystemParam,)]
struct StarSpawnResources<'w,> {
    star_config: Res<'w, StarConfig,>,
    boundary:    Res<'w, Boundary,>,
    meshes:      ResMut<'w, Assets<Mesh,>,>,
    materials:   ResMut<'w, Assets<StandardMaterial,>,>,
}

fn replace_stars(
    mut commands: Commands,
    mut timer: ResMut<StarReplaceTimer,>,
    time: Res<Time,>,
    stars: Query<Entity, With<Stars,>,>,
    mut resources: StarSpawnResources,
) {
    if timer.0.tick(time.delta(),).just_finished()
        && resources.star_config.star_count == stars.iter().count()
    {
        let stars_to_spawn = resources.star_config.batch_size_replace;

        // Collect all star entities into a vector
        let mut all_stars: Vec<Entity,> = stars.iter().collect();

        // Shuffle the vector randomly
        all_stars.shuffle(&mut thread_rng(),);

        // Take the first `stars_to_spawn` stars from the shuffled vector
        for &entity in all_stars.iter().take(stars_to_spawn,) {
            despawn(&mut commands, entity,);

            spawn_star(
                &mut commands,
                &resources.star_config,
                &resources.boundary,
                &mut resources.meshes,
                &mut resources.materials,
                1,
            );
        }
    }
}

// generate BATCH_SIZE stars at a time until you get to the desired number of
// stars this will fill them in as the program starts and looks cooler that way
// otherwise the system blocks on spawning that many stars at startup
fn spawn_star_tasks(
    mut commands: Commands,
    mut timer: ResMut<StarSpawnTimer,>,
    time: Res<Time,>,
    mut spawned_count: Local<usize,>,
    mut resources: StarSpawnResources,
) {
    let star_config = resources.star_config;

    if timer.0.tick(time.delta(),).just_finished() && *spawned_count < star_config.star_count {
        let stars_to_spawn =
            (star_config.star_count - *spawned_count).min(star_config.batch_size_spawn,);
        // println!("stars_to_spawn {}", stars_to_spawn);

        spawn_star(
            &mut commands,
            &star_config,
            &resources.boundary,
            &mut resources.meshes,
            &mut resources.materials,
            stars_to_spawn,
        );

        *spawned_count += stars_to_spawn;
    }
}

fn spawn_star(
    commands: &mut Commands,
    star_config: &Res<StarConfig,>,
    boundary: &Res<Boundary,>,
    meshes: &mut ResMut<Assets<Mesh,>,>,
    materials: &mut ResMut<Assets<StandardMaterial,>,>,
    stars_to_spawn: usize,
) {
    let longest_diagonal = boundary.longest_diagonal;
    let inner_sphere_radius = longest_diagonal + star_config.star_field_inner_diameter;
    let outer_sphere_radius = inner_sphere_radius + star_config.star_field_outer_diameter;

    for _ in 0..stars_to_spawn {
        let mut rng = rand::thread_rng();
        let point = {
            let u: f32 = rng.gen_range(0.0..1.0,);
            let v: f32 = rng.gen_range(0.0..1.0,);
            let theta = u * std::f32::consts::PI * 2.0;
            let phi = (2.0 * v - 1.0).acos();
            let r = rng.gen_range(inner_sphere_radius..outer_sphere_radius,);

            let x = r * theta.cos() * phi.sin();
            let y = r * theta.sin() * phi.sin();
            let z = r * phi.cos();

            Vec3::new(x, y, z,)
        };

        // Increase the likelihood of generating higher values for R, G, B
        let emissive_r = rng.gen_range(8.0..15.0,);
        let emissive_g = rng.gen_range(8.0..15.0,);
        let emissive_b = rng.gen_range(8.0..15.0,);
        let emissive_a = rng.gen_range(8.0..15.0,);

        let transform = Transform::from_translation(point,);

        let material = materials.add(StandardMaterial {
            emissive: LinearRgba::new(emissive_r, emissive_g, emissive_b, emissive_a,),
            ..default()
        },);

        let min = star_config.star_radius / 10.;

        let radius = rng.gen_range(min..star_config.star_radius,);
        let star_mesh_handle = meshes.add(Sphere::new(radius,).mesh(),);

        commands
            .spawn(PbrBundle {
                mesh: star_mesh_handle.clone(),
                material,
                transform,
                ..default()
            },)
            .insert(Stars,)
            .insert(RenderLayers::from_layers(RenderLayer::Stars.layers(),),);
    }
}
