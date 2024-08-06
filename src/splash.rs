use crate::stars::GAME_LAYER;
use crate::state::GameState;
use bevy::prelude::*;
use bevy::render::view::RenderLayers;

const SPLASH_TIMER_SECONDS: f32 = 2.;

pub(crate) struct SplashPlugin;

#[derive(Component)]
pub(crate) struct SplashText;

#[derive(Resource, Debug)]
struct SplashTimer {
    pub timer: Timer,
}

impl Plugin for SplashPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SplashTimer {
            timer: Timer::from_seconds(SPLASH_TIMER_SECONDS, TimerMode::Once),
        })
        .add_systems(Startup, splash_screen)
        .add_systems(Update, run_splash.run_if(in_state(GameState::Splash)));
    }
}

fn splash_screen(mut commands: Commands) {
    let splash_text = Text::from_section(
        // Accepts a String or any type that converts into a String, such as &str.
        "nateroids",
        TextStyle {
            font_size: 1.0,
            ..default()
        },
    );

    let splash_style = Style {
        align_self: AlignSelf::Center,
        justify_self: JustifySelf::Center,
        ..default()
    };

    let mut press_space_style = splash_style.clone();
    press_space_style.top = Val::Px(50.0);

    commands.spawn((
        TextBundle {
            text: splash_text,
            style: splash_style,
            ..default()
        },
        RenderLayers::layer(GAME_LAYER),
        SplashText,
    ));
}

fn run_splash(
    mut next_state: ResMut<NextState<GameState>>,
    mut spawn_timer: ResMut<SplashTimer>,
    time: Res<Time>,
    mut q_text: Query<&mut Text, With<SplashText>>,
) {
    spawn_timer.timer.tick(time.delta());
    if let Ok(mut text) = q_text.get_single_mut() {
        text.sections[0].style.font_size += 1.0;
    }
    if spawn_timer.timer.just_finished() {
        next_state.set(GameState::InGame);
    }
}
