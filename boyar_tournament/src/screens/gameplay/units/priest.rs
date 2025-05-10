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
    app.add_observer(spawn_priest);

    app.configure_loading_state(
        LoadingStateConfig::new(GameState::Loading).load_collection::<PriestAssets>(),
    );
}

#[derive(Event)]
pub struct SpawnPriest(pub Entity, pub ArenaPos, pub PlayerNumber);

#[derive(Component)]
#[require(
    Health,

    Name(|| Name::new("Жрец")),
    DynamicScale(|| DynamicScale(0.55)),
    UnitState,
    ArenaHeightOffset(|| ArenaHeightOffset(1.1)),
)]
struct Priest;

#[derive(Resource, AssetCollection)]
struct PriestAssets {
    #[asset(path = "units/priest/priest.aseprite")]
    sprite: Handle<Aseprite>,
}

fn spawn_priest(
    trigger: Trigger<SpawnPriest>,
    mut cmd: Commands,
    self_num: Res<PlayerNumber>,
    assets: ResMut<PriestAssets>,
    mut network_mapping: ResMut<NetworkMapping>,
) {
    let &SpawnPriest(entity, pos, player_num) = trigger.event();

    let direction = self_num.spawn_direction(player_num);

    let priest = cmd
        .spawn((
            Priest,
            pos,
            direction,
            AseSpriteAnimation {
                animation: Animation::tag(direction.tag()),
                aseprite: assets.sprite.clone(),
            },
        ))
        .id();

    network_mapping.insert(entity, priest);
}
