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
    app.add_observer(spawn_giant);

    app.configure_loading_state(
        LoadingStateConfig::new(GameState::Loading).load_collection::<GiantAssets>(),
    );
}

#[derive(Event)]
pub struct SpawnGiant(pub Entity, pub ArenaPos, pub PlayerNumber);

#[derive(Component)]
#[require(
    Health,

    Name(|| Name::new("Гигант")),
    DynamicScale(|| DynamicScale(1.)),
    UnitState,
    ArenaHeightOffset(|| ArenaHeightOffset(3.)),
)]
struct Giant;

#[derive(Resource, AssetCollection)]
struct GiantAssets {
    #[asset(path = "units/giant/giant.aseprite")]
    sprite: Handle<Aseprite>,
}

fn spawn_giant(
    trigger: Trigger<SpawnGiant>,
    mut cmd: Commands,
    self_num: Res<PlayerNumber>,
    assets: ResMut<GiantAssets>,
    mut network_mapping: ResMut<NetworkMapping>,
) {
    let &SpawnGiant(entity, pos, player_num) = trigger.event();

    let direction = self_num.spawn_direction(player_num);

    let giant = cmd
        .spawn((
            Giant,
            pos,
            direction,
            AseSpriteAnimation {
                animation: Animation::tag(direction.tag()),
                aseprite: assets.sprite.clone(),
            },
        ))
        .id();

    network_mapping.insert(entity, giant);
}
