use bevy::prelude::*;
use bevy_aseprite_ultra::prelude::*;
use bevy_asset_loader::prelude::*;
use common::{ArenaPos, Health, PlayerNumber, UnitState};

use crate::{
    scaling::DynamicScale,
    screens::{
        gameplay::{arena::ArenaHeightOffset, networking::NetworkMapping},
        GameState,
    },
};

use super::{IntoTag, SpawnDirection};

pub(super) fn plugin(app: &mut App) {
    app.add_observer(spawn_bat);

    app.configure_loading_state(
        LoadingStateConfig::new(GameState::Loading).load_collection::<BatAssets>(),
    );
}

#[derive(Event)]
pub struct SpawnBat(pub Entity, pub ArenaPos, pub PlayerNumber);

#[derive(Component)]
#[require(
    Health,

    Name(|| Name::new("Мышь")),
    DynamicScale(|| DynamicScale(0.3)),
    UnitState,
    ArenaHeightOffset(|| ArenaHeightOffset(2.5)),
)]
struct Bat;

#[derive(Resource, AssetCollection)]
struct BatAssets {
    #[asset(path = "units/bat/bat.aseprite")]
    sprite: Handle<Aseprite>,
}

fn spawn_bat(
    trigger: Trigger<SpawnBat>,
    mut cmd: Commands,
    self_num: Res<PlayerNumber>,
    assets: ResMut<BatAssets>,
    mut network_mapping: ResMut<NetworkMapping>,
) {
    let &SpawnBat(entity, pos, player_num) = trigger.event();

    let direction = self_num.spawn_direction(player_num);

    let bat = cmd
        .spawn((
            Bat,
            pos,
            direction,
            AseSpriteAnimation {
                animation: Animation::tag(direction.tag()),
                aseprite: assets.sprite.clone(),
            },
        ))
        .id();

    network_mapping.insert(entity, bat);
}
