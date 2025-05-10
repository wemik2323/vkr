#[cfg(debug_assertions)]
mod dev_tools;
mod scaling;
mod screens;

use bevy::{audio::Volume, prelude::*};

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_camera);

        app.insert_resource(GlobalVolume {
            volume: Volume::new(0.3),
        });

        app.add_plugins((scaling::plugin, screens::plugin));

        #[cfg(debug_assertions)]
        app.add_plugins(dev_tools::plugin);
    }
}

fn spawn_camera(mut cmd: Commands) {
    cmd.spawn((Camera2d, IsDefaultUiCamera));
}
