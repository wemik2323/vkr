// Выключить коммандную строку в релиз-сборках для Windows
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use bevy::{asset::AssetMetaCheck, prelude::*, window::PrimaryWindow, winit::WinitWindows};
use boyar_tournament::GamePlugin;
use std::io::Cursor;
use winit::window::Icon;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Window {
                        title: "Боярский Турнир".into(),
                        ..default()
                    }
                    .into(),
                    ..default()
                })
                .set(ImagePlugin::default_nearest())
                .set(AssetPlugin {
                    meta_check: AssetMetaCheck::Never,
                    file_path: "../assets".into(),
                    processed_file_path: "../assets".into(),
                    ..default()
                }),
        )
        .add_plugins(GamePlugin)
        .add_systems(Startup, set_window_icon)
        .run();
}

fn set_window_icon(
    windows: NonSend<WinitWindows>,
    primary_window: Query<Entity, With<PrimaryWindow>>,
) {
    let primary_window = primary_window.single();
    let Some(primary_window) = windows.get_window(primary_window) else {
        return;
    };
    let icon_buf = Cursor::new(include_bytes!("../../assets/icons/desktop_icon.png"));
    if let Ok(image) = image::load(icon_buf, image::ImageFormat::Png) {
        let image = image.into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        let icon = Icon::from_rgba(rgba, width, height).unwrap();
        primary_window.set_window_icon(Some(icon));
    }
}
