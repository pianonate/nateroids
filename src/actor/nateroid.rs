use crate::{
    actor::{
        actor_config::{
            spawn_actor,
            ActorType,
            EnsembleConfig,
        },
    },
    asset_loader::SceneAssets,
    boundary::{
        Boundary,
    },
   
    schedule::InGameSet,
};
use bevy::{
    prelude::*,
};

pub struct NateroidPlugin;

impl Plugin for NateroidPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, spawn_nateroid.in_set(InGameSet::EntityUpdates));
    }
}

fn spawn_nateroid(
    mut commands: Commands,
    mut ensemble_config: ResMut<EnsembleConfig>,
    boundary: Res<Boundary>,
    scene_assets: Res<SceneAssets>,
    time: Res<Time>,
) {
    
    let nateroid_config = &mut ensemble_config.nateroid;
    if !nateroid_config.spawnable {
        return
    }

    let spawn_timer = nateroid_config.spawn_timer.as_mut().unwrap();
    spawn_timer.tick(time.delta());

    if !spawn_timer.just_finished() {
        return
    }
        
    let nateroid_actor_config = ensemble_config.get_actor_config(ActorType::Nateroid);
    
    spawn_actor(
        &mut commands,
        nateroid_actor_config,
        scene_assets.nateroid.clone(),
        None,
        boundary,
    );
}