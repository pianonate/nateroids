use bevy::prelude::KeyCode::Escape;
use bevy::prelude::*;
use bevy_rapier3d::plugin::RapierConfiguration;

#[cfg(feature = "inspectors")]
use bevy_inspector_egui::quick::StateInspectorPlugin;

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, Reflect, States)]
pub enum GameState {
    #[default]
    InGame,
    Paused,
    GameOver,
}

pub struct StatePlugin;

impl Plugin for StatePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>()
            .add_systems(
                Update,
                (
                    game_state_input_events,
                    transition_to_in_game.run_if(in_state(GameState::GameOver)),
                ),
            )
            .add_systems(OnEnter(GameState::Paused), pause_rapier)
            .add_systems(OnEnter(GameState::InGame), unpause_rapier);

        #[cfg(feature = "inspectors")]
        app.register_type::<GameState>()
            .add_plugins(StateInspectorPlugin::<GameState>::default());
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
) {
    if keyboard_input.just_pressed(Escape) {
        match state.get() {
            GameState::InGame => {
                next_state.set(GameState::Paused);
            }
            GameState::Paused => {
                next_state.set(GameState::InGame);
            }
            _ => (),
        }
    }
}

fn transition_to_in_game(mut next_state: ResMut<NextState<GameState>>) {
    next_state.set(GameState::InGame);
}

fn pause_rapier(mut rapier_config: ResMut<RapierConfiguration>) {
    rapier_config.physics_pipeline_active = false;
}

fn unpause_rapier(mut rapier_config: ResMut<RapierConfiguration>) {
    rapier_config.physics_pipeline_active = true;
}
