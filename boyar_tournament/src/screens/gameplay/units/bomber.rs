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
    app.add_observer(spawn_bomber);

    app.configure_loading_state(
        LoadingStateConfig::new(GameState::Loading).load_collection::<BomberAssets>(),
    );
}

#[derive(Event)]
pub struct SpawnBomber(pub Entity, pub ArenaPos, pub PlayerNumber);

#[derive(Component)]
#[require(
    Health,

    Name(|| Name::new("Подрывник")),
    DynamicScale(|| DynamicScale(0.7)),
    UnitState,
    ArenaHeightOffset(|| ArenaHeightOffset(2.3)),
)]
struct Bomber;

#[derive(Resource, AssetCollection)]
struct BomberAssets {
    #[asset(path = "units/bomber/bomber.aseprite")]
    sprite: Handle<Aseprite>,
}

fn spawn_bomber(
    trigger: Trigger<SpawnBomber>,
    mut cmd: Commands,
    self_num: Res<PlayerNumber>,
    assets: ResMut<BomberAssets>,
    mut network_mapping: ResMut<NetworkMapping>,
) {
    let &SpawnBomber(entity, pos, player_num) = trigger.event();

    let direction = self_num.spawn_direction(player_num);

    let bomber = cmd
        .spawn((
            Bomber,
            pos,
            direction,
            AseSpriteAnimation {
                animation: Animation::tag(direction.tag()),
                aseprite: assets.sprite.clone(),
            },
        ))
        .id();

    network_mapping.insert(entity, bomber);
}
