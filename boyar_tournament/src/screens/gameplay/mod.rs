use bevy::prelude::*;
use bevy_aseprite_ultra::prelude::*;
use bevy_asset_loader::prelude::*;

use crate::scaling::{DynamicScale, DynamicTransform};

use super::GameState;

mod arena;
mod deck;
mod networking;
mod projectiles;
mod units;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(AsepriteUltraPlugin);

    app.add_plugins((
        arena::plugin,
        networking::plugin,
        units::plugin,
        deck::plugin,
        projectiles::plugin,
    ));

    app.configure_loading_state(
        LoadingStateConfig::new(GameState::Loading).load_collection::<FontAssets>(),
    );
}

#[derive(AssetCollection, Resource)]
struct FontAssets {
    #[asset(path = "Keleti-Regular.ttf")]
    font: Handle<Font>,
}

fn spawn_text(
    cmd: &mut Commands,
    text: &str,
    font: Handle<Font>,
    font_size: f32,
    color: Color,
    dynamic_scale: f32,
    dynamic_transform: (f32, f32),
    state: GameState,
) {
    cmd.spawn((
        Text2d::new(text),
        TextFont::from_font(font.clone()).with_font_size(font_size),
        TextColor(color),
        StateScoped(state),
        DynamicScale(dynamic_scale),
        DynamicTransform(dynamic_transform.0, dynamic_transform.1),
    ))
    .insert(Transform::from_xyz(0., 0., 0.2));

    cmd.spawn((
        Text2d::new(text),
        TextFont::from_font(font.clone()).with_font_size(font_size),
        TextColor(Color::BLACK),
        StateScoped(state),
        DynamicScale(dynamic_scale),
        DynamicTransform(dynamic_transform.0 + 0.03, dynamic_transform.1 - 0.03),
    ))
    .insert(Transform::from_xyz(0., 0., 0.1));
}
