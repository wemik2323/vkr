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
    app.add_observer(spawn_rus);

    app.configure_loading_state(
        LoadingStateConfig::new(GameState::Loading).load_collection::<RusAssets>(),
    );
}

#[derive(Event)]
pub struct SpawnRus(pub Entity, pub ArenaPos, pub PlayerNumber);

#[derive(Component)]
#[require(
    Health,

    Name(|| Name::new("Рус")),
    DynamicScale(|| DynamicScale(0.6)),
    UnitState,
    ArenaHeightOffset(|| ArenaHeightOffset(2.)),
)]
struct Rus;

#[derive(Resource, AssetCollection)]
struct RusAssets {
    #[asset(path = "units/rus/rus.aseprite")]
    sprite: Handle<Aseprite>,
}

fn spawn_rus(
    trigger: Trigger<SpawnRus>,
    mut cmd: Commands,
    self_num: Res<PlayerNumber>,
    assets: ResMut<RusAssets>,
    mut network_mapping: ResMut<NetworkMapping>,
) {
    let &SpawnRus(entity, pos, player_num) = trigger.event();

    let direction = self_num.spawn_direction(player_num);

    let rus = cmd
        .spawn((
            Rus,
            pos,
            direction,
            AseSpriteAnimation {
                animation: Animation::tag(direction.tag()),
                aseprite: assets.sprite.clone(),
            },
        ))
        .id();

    network_mapping.insert(entity, rus);
}
