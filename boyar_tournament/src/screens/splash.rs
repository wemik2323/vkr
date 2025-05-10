use bevy::prelude::*;

use crate::scaling::DynamicScale;

use super::GameState;

pub(super) fn plugin(app: &mut App) {
    app.insert_resource(ClearColor(Color::BLACK));
    app.add_systems(OnEnter(GameState::Splash), spawn_splash_screen);

    app.add_systems(
        Update,
        update_fade_in_out.run_if(in_state(GameState::Splash)),
    );

    app.add_systems(OnEnter(GameState::Splash), insert_splash_timer);
    app.add_systems(OnExit(GameState::Splash), remove_splash_timer);
    app.add_systems(
        Update,
        update_splash_timer.run_if(in_state(GameState::Splash)),
    );
}

const SPLASH_DURATION_SEC: f32 = 1.5;
const SPLASH_FADE_DURATION_SEC: f32 = 0.75;

fn spawn_splash_screen(mut cmd: Commands, asset_server: Res<AssetServer>) {
    cmd.spawn((
        Sprite {
            image: asset_server.load("screens/splash/valetoriy.png"),
            ..default()
        },
        ImageFadeInOut {
            total_duration_sec: SPLASH_DURATION_SEC,
            fade_duration_sec: SPLASH_FADE_DURATION_SEC,
            t: 0.,
        },
        StateScoped(GameState::Splash),
        DynamicScale(0.5),
    ));
    cmd.spawn((
        AudioPlayer::new(asset_server.load("screens/splash/splash.ogg")),
        PlaybackSettings::DESPAWN,
    ));
}

#[derive(Component)]
struct ImageFadeInOut {
    total_duration_sec: f32,
    fade_duration_sec: f32,
    /// Текущий прогресс от 0 до total_duration_sec
    t: f32,
}

impl ImageFadeInOut {
    fn alpha(&self) -> f32 {
        let t = (self.t / self.total_duration_sec).clamp(0.0, 1.0);
        let fade = self.fade_duration_sec / self.total_duration_sec;

        // Трапезоидный график прозрачности
        ((1.0 - (2.0 * t - 1.0).abs()) / fade).min(1.0)
    }
}

fn update_fade_in_out(
    time: Res<Time>,
    mut animation_query: Query<(&mut ImageFadeInOut, &mut Sprite)>,
) {
    let (mut anim, mut sprite) = animation_query.single_mut();
    anim.t += time.delta_secs();
    sprite.color.set_alpha(anim.alpha())
}

#[derive(Resource, Debug, Clone, PartialEq)]
struct SplashTimer(Timer);

fn insert_splash_timer(mut cmd: Commands) {
    cmd.insert_resource(SplashTimer(Timer::from_seconds(
        SPLASH_DURATION_SEC,
        TimerMode::Once,
    )));
}

fn remove_splash_timer(mut cmd: Commands) {
    cmd.remove_resource::<SplashTimer>();
}

fn update_splash_timer(
    time: Res<Time>,
    mut timer: ResMut<SplashTimer>,
    mut next_screen: ResMut<NextState<GameState>>,
) {
    timer.0.tick(time.delta());

    if timer.0.just_finished() {
        next_screen.set(GameState::Loading);
    }
}
