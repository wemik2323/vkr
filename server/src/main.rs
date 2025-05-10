use bevy::{log::LogPlugin, prelude::*};

mod ai;
mod networking;
mod projectiles;
mod units;

fn main() {
    App::new()
        .add_plugins((
            MinimalPlugins,
            LogPlugin::default(),
            ai::plugin,
            units::plugin,
            projectiles::plugin,
            networking::plugin,
        ))
        .run();
}
