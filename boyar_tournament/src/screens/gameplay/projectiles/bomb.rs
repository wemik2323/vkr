use bevy::prelude::*;
use bevy_aseprite_ultra::prelude::*;
use bevy_asset_loader::prelude::*;
use common::ArenaPos;

use crate::{
    scaling::DynamicScale,
    screens::{
        gameplay::{arena::ArenaHeightOffset, networking::NetworkMapping},
        GameState,
    },
};

use super::ProjectileTargets;

pub(super) fn plugin(app: &mut App) {
    app.add_observer(spawn_bomb);

    app.configure_loading_state(
        LoadingStateConfig::new(GameState::Loading).load_collection::<BombAssets>(),
    );
}

#[derive(Event)]
pub struct SpawnBomb(pub Entity, pub Entity, pub Entity, pub ArenaPos);

#[derive(Component)]
#[require(
    Name(|| Name::new("Бомба")),
    DynamicScale(|| DynamicScale(0.5)),
    ArenaHeightOffset(|| ArenaHeightOffset(0.3)),
)]
struct Bomb;

#[derive(Resource, AssetCollection)]
struct BombAssets {
    #[asset(path = "units/bomber/bomb.aseprite")]
    sprite: Handle<Aseprite>,
}

fn spawn_bomb(
    trigger: Trigger<SpawnBomb>,
    mut cmd: Commands,
    assets: ResMut<BombAssets>,
    mut network_mapping: ResMut<NetworkMapping>,
) {
    let &SpawnBomb(entity, attacker, receiver, pos) = trigger.event();
    let Some(attacker) = network_mapping.get(&attacker) else {
        return;
    };
    let Some(receiver) = network_mapping.get(&receiver) else {
        return;
    };

    let bomb = cmd
        .spawn((
            Bomb,
            pos,
            AseSpriteAnimation {
                animation: Animation::tag("bomb"),
                aseprite: assets.sprite.clone(),
            },
            ProjectileTargets(*attacker, *receiver, 0.5),
        ))
        .id();

    network_mapping.insert(entity, bomb);
}
