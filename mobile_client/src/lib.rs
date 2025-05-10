use bevy::prelude::*;
use bevy::window::{AppLifecycle, WindowMode};
use bevy::winit::WinitSettings;
use boyar_tournament::GamePlugin;

#[bevy_main]
fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Window {
                        resizable: false,
                        mode: WindowMode::BorderlessFullscreen(MonitorSelection::Primary),
                        ..default()
                    }
                    .into(),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
            GamePlugin,
        ))
        .insert_resource(WinitSettings::mobile())
        .add_systems(Update, handle_lifetime)
        .run();
}

/// Остановка звука при переходе приложения в фоновый режим
// Взято из официального android примера, не проверял нужно или нет
fn handle_lifetime(
    mut lifecycle_events: EventReader<AppLifecycle>,
    music_controller: Single<&AudioSink>,
) {
    for event in lifecycle_events.read() {
        match event {
            AppLifecycle::Idle | AppLifecycle::WillSuspend | AppLifecycle::WillResume => {}
            AppLifecycle::Suspended => music_controller.pause(),
            AppLifecycle::Running => music_controller.play(),
        }
    }
}
