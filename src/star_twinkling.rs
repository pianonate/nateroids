use crate::config::StarConfig;
use bevy::{prelude::*, render::view::RenderLayers};
use rand::{
    seq::SliceRandom,
    {thread_rng, Rng},
};

#[derive(Component)]
struct Twinkling {
    original_emissive: Vec4,
    target_emissive: Vec4,
    timer: Timer,
}

pub struct StarTwinklingPlugin;

impl Plugin for StarTwinklingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<StarConfig>()
            .add_systems(Update, (start_twinkling, update_twinkling));
    }
}

fn start_twinkling(
    mut commands: Commands,
    stars: Query<(Entity, &Handle<StandardMaterial>), (With<RenderLayers>, Without<Twinkling>)>,
    materials: Res<Assets<StandardMaterial>>,
    twinkle_config: Res<StarConfig>,
    star_config: Res<StarConfig>,
) {
    let mut rng = thread_rng();

    // Collect stars into a vector and shuffle it
    let mut stars_vec: Vec<_> = stars.iter().collect();
    stars_vec.shuffle(&mut rng);

    let take_count = (star_config.star_count as f32 * twinkle_config.twinkle_sample_rate) as usize;
    let min_intensity = twinkle_config.min_intensity;
    let max_intensity = twinkle_config.max_intensity;

    // Take the first 50 elements from the shuffled vector
    for (entity, material_handle) in stars_vec.into_iter().take(take_count) {
        if rng.gen::<f32>() < twinkle_config.twinkle_chance {
            if let Some(material) = materials.get(material_handle) {
                let original_emissive = Vec4::new(
                    material.emissive.red,
                    material.emissive.green,
                    material.emissive.blue,
                    material.emissive.alpha,
                );
                let intensity = rng.gen_range(min_intensity..max_intensity);
                let target_emissive = original_emissive * intensity;
                let duration =
                    rng.gen_range(twinkle_config.min_duration..twinkle_config.max_duration);

                commands.entity(entity).insert(Twinkling {
                    original_emissive,
                    target_emissive,
                    timer: Timer::from_seconds(duration, TimerMode::Once),
                });
            }
        }
    }
}

fn update_twinkling(
    mut commands: Commands,
    time: Res<Time>,
    mut stars: Query<(Entity, &Handle<StandardMaterial>, &mut Twinkling)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut star_count = 0;
    for (entity, material_handle, mut twinkling) in stars.iter_mut() {
        twinkling.timer.tick(time.delta());

        if let Some(material) = materials.get_mut(material_handle) {
            let progress =
                twinkling.timer.elapsed_secs() / twinkling.timer.duration().as_secs_f32();
            // .5 -> brighten until halfway and then dim back
            // .2 in the lerp -> used to make sure full range of the lerp function is used in each
            //                   half of the animation - so we go from 0..1 in each half
            let new_emissive = if progress < 0.5 {
                twinkling
                    .original_emissive
                    .lerp(twinkling.target_emissive, progress * 2.0)
            } else {
                twinkling
                    .target_emissive
                    .lerp(twinkling.original_emissive, (progress - 0.5) * 2.0)
            };
            material.emissive = LinearRgba::new(
                new_emissive.x,
                new_emissive.y,
                new_emissive.z,
                new_emissive.w,
            );
        }

        if twinkling.timer.finished() {
            commands.entity(entity).remove::<Twinkling>();
        }

        star_count += 1;
    }

    println!("twinklers: {}", star_count)
}
