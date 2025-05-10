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
    app.add_observer(spawn_musketeer);

    app.configure_loading_state(
        LoadingStateConfig::new(GameState::Loading).load_collection::<MusketeerAssets>(),
    );
}

#[derive(Event)]
pub struct SpawnMusketeer(pub Entity, pub ArenaPos, pub PlayerNumber);

#[derive(Component)]
#[require(
    Health,

    Name(|| Name::new("Стрелок")),
    DynamicScale(|| DynamicScale(0.55)),
    UnitState,
    ArenaHeightOffset(|| ArenaHeightOffset(1.3)),
)]
struct Musketeer;

#[derive(Resource, AssetCollection)]
struct MusketeerAssets {
    #[asset(path = "units/musketeer/musketeer.aseprite")]
    sprite: Handle<Aseprite>,
}

fn spawn_musketeer(
    trigger: Trigger<SpawnMusketeer>,
    mut cmd: Commands,
    self_num: Res<PlayerNumber>,
    assets: ResMut<MusketeerAssets>,
    mut network_mapping: ResMut<NetworkMapping>,
) {
    let &SpawnMusketeer(entity, pos, player_num) = trigger.event();

    let direction = self_num.spawn_direction(player_num);

    let musketeer = cmd
        .spawn((
            Musketeer,
            pos,
            direction,
            AseSpriteAnimation {
                animation: Animation::tag(direction.tag()),
                aseprite: assets.sprite.clone(),
            },
        ))
        .id();

    network_mapping.insert(entity, musketeer);
}
