use bevy::prelude::*;

mod gameplay;
mod loading;
mod splash;
mod ui;

pub(super) fn plugin(app: &mut App) {
    app.init_state::<GameState>();
    app.enable_state_scoped_entities::<GameState>();

    app.add_plugins((
        splash::plugin,
        loading::plugin,
        gameplay::plugin,
        ui::plugin,
    ));
}

#[derive(States, Debug, PartialEq, Eq, Clone, Hash, Default, Copy)]
pub enum GameState {
    #[default]
    Splash,
    Loading,
    // Menu,
    Gameplay,
}
