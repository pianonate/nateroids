use bevy::prelude::KeyCode::Escape;
use bevy::prelude::*;
use bevy_rapier3d::plugin::RapierConfiguration;

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum GameState {
    #[default]
    InGame,
    Paused,
    GameOver,
}

pub struct StatePlugin;

impl Plugin for StatePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>().add_systems(
            Update,
            (
                game_state_input_events,
                transition_to_in_game.run_if(in_state(GameState::GameOver)),
            ),
        );
    }

    fn name(&self) -> &str {
        "state plugin"
    }
}

// i think it would be a lot of trouble to merge rapier's schedule
// with this one so i'm just going to pause it directly
// with its physics_pipeline_active configuration flag
fn game_state_input_events(
    mut next_state: ResMut<NextState<GameState>>,
    state: Res<State<GameState>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut rapier_config: ResMut<RapierConfiguration>,
) {
    if keyboard_input.just_pressed(Escape) {
        match state.get() {
            GameState::InGame => {
                next_state.set(GameState::Paused);
                rapier_config.physics_pipeline_active = false;
            },
            GameState::Paused => {
                next_state.set(GameState::InGame);
                rapier_config.physics_pipeline_active = true;
            },
            _ => (),
        }
    }
}

fn transition_to_in_game(mut next_state: ResMut<NextState<GameState>>) {
    next_state.set(GameState::InGame);
}
