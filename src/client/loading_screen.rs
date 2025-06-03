use bevy::prelude::*;

use crate::GameState;

#[derive(Component)]
pub struct LoadingView;

#[derive(Component)]
pub struct LoadingScreenFade {
    pub timer: Timer,
    pub duration: f32,
    pub start_alpha: f32,
    pub end_alpha: f32,
}

pub struct LoadingScreenPlugin;

impl Plugin for LoadingScreenPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::LoadingMap), begin_loading)
            .add_systems(OnExit(GameState::LoadingMap), end_loading)
            .add_systems(Update, animate_loading_screen_fade);
    }
}

fn begin_loading(mut commands: Commands) {
    commands
        .spawn((
            LoadingView,
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default() // ...
            },
            BackgroundColor(Color::srgba(0., 0., 0., 0.)),
            LoadingScreenFade {
                timer: Timer::from_seconds(0.3, TimerMode::Once),
                duration: 0.3,
                start_alpha: 0.,
                end_alpha: 1.,
            },
        ))
        .with_child(Text::new("Loading..."));
}

fn end_loading(
    mut commands: Commands,
    existing_fade_query: Query<(Entity, &LoadingScreenFade), With<LoadingView>>,
    loading_screen: Query<(Entity, &BackgroundColor), With<LoadingView>>,
) {
    for (entity, _) in &existing_fade_query {
        commands.entity(entity).remove::<LoadingScreenFade>();
    }
    for (v, background) in &loading_screen {
        commands.entity(v).insert(LoadingScreenFade {
            timer: Timer::from_seconds(0.3, TimerMode::Once),
            duration: 0.3,
            start_alpha: background.0.alpha(),
            end_alpha: 0.,
        });
    }
}

fn animate_loading_screen_fade(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut BackgroundColor, &mut LoadingScreenFade)>,
) {
    for (entity, mut bg_color, mut fade) in &mut query {
        fade.timer.tick(time.delta());

        let progress = fade.timer.elapsed_secs() / fade.duration;
        let alpha = fade.start_alpha.lerp(fade.end_alpha, progress);

        bg_color.0 = Color::srgba(0.0, 0.0, 0.0, alpha);

        if fade.timer.finished() {
            if fade.end_alpha <= 0.0 {
                commands.entity(entity).despawn_recursive();
            } else {
                // Remove component when done to stop updating
                commands.entity(entity).remove::<LoadingScreenFade>();
            }
        }
    }
}
