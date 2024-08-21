use crate::{
    actor::{
        actor_spawner::spawn_actor,
        actor_template::SpaceshipConfig,
        spaceship_control::SpaceshipControl,
    },
    playfield::Boundary,
    schedule::InGameSet,
    state::GameState,
};
use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

#[derive(Component, Debug)]
pub struct Spaceship;

#[derive(Component, Default)]
pub struct ContinuousFire;

pub struct SpaceshipPlugin;
impl Plugin for SpaceshipPlugin {
    // make sure this is done after asset_loader has run
    fn build(&self, app: &mut App) {
        // we can enter InGame a couple of ways - when we do, spawn a spaceship
        app.add_systems(OnExit(GameState::Splash), spawn_spaceship)
            .add_systems(OnExit(GameState::GameOver), spawn_spaceship)
            // check if spaceship is destroyed...this will change the GameState
            .add_systems(Update, spaceship_destroyed.in_set(InGameSet::EntityUpdates));
    }
}

//todo: #bug - you don't need to bring boundary in except for nateroids
// spawning
// - make it optional
fn spawn_spaceship(mut commands: Commands, spaceship_config: Res<SpaceshipConfig>, boundary: Res<Boundary>) {
    if !spaceship_config.0.spawnable {
        return;
    }

    let spaceship_input = InputManagerBundle::with_map(SpaceshipControl::generate_input_map());

    spawn_actor(&mut commands, &spaceship_config.0, None, boundary)
        .insert(spaceship_input)
        .insert(Spaceship);
}

// check if spaceship exists or not - query
// if get_single() (there should only be one - returns an error then the
// spaceship doesn't exist
fn spaceship_destroyed(
    mut next_state: ResMut<NextState<GameState>>,
    query: Query<Entity, With<Spaceship>>,
    state: Res<State<GameState>>,
) {
    if query.get_single().is_err() {
        println!(
            "spaceship destroyed: {:?}, count {:?}",
            state,
            query.iter().count()
        );
        next_state.set(GameState::GameOver);
    }
}
