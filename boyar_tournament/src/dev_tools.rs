use bevy::{
    dev_tools::{
        fps_overlay::FpsOverlayPlugin,
        states::log_transitions,
        ui_debug_overlay::{DebugUiPlugin, UiDebugOptions},
    },
    input::common_conditions::input_just_pressed,
    prelude::*,
};
use bevy_inspector_egui::quick::WorldInspectorPlugin;

use crate::screens::GameState;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Update, log_transitions::<GameState>);

    app.add_plugins(FpsOverlayPlugin::default());

    app.add_plugins(WorldInspectorPlugin::new());

    app.add_plugins(DebugUiPlugin);
    app.add_systems(
        Update,
        toggle_debug_ui.run_if(input_just_pressed(KeyCode::Backquote)),
    );
}

fn toggle_debug_ui(mut options: ResMut<UiDebugOptions>) {
    options.toggle();
}
