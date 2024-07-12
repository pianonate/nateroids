use bevy::prelude::*;
use bevy::prelude::KeyCode::Escape;

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum GameState {
    #[default]
    InGame,
    Paused,
    GameOVer,
}

pub struct StatePlugin;

impl  Plugin for StatePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>()
            .add_systems(Update, game_state_input_events);
    }

    fn name(&self) -> &str {
        "state plugin"
    }
}

fn  game_state_input_events(mut next_state: ResMut<NextState<GameState>>, state: Res<State<GameState>>, keyboard_input: Res<ButtonInput<KeyCode>>) {
    if keyboard_input.just_pressed(Escape) {
        match state.get() {
            GameState::InGame => next_state.set(GameState::Paused),
            GameState::Paused => next_state.set(GameState::InGame),
            _ => ()
        }
    }
}
