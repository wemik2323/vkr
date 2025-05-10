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
    app.add_observer(spawn_bullet);

    app.configure_loading_state(
        LoadingStateConfig::new(GameState::Loading).load_collection::<BulletAssets>(),
    );
}

#[derive(Event)]
pub struct SpawnBullet(pub Entity, pub Entity, pub Entity, pub ArenaPos);

#[derive(Component)]
#[require(
    Name(|| Name::new("Пуля")),
    DynamicScale(|| DynamicScale(1.)),
    ArenaHeightOffset(|| ArenaHeightOffset(0.)),
)]
struct Bullet;

#[derive(Resource, AssetCollection)]
struct BulletAssets {
    #[asset(path = "units/musketeer/bullet.aseprite")]
    sprite: Handle<Aseprite>,
}

fn spawn_bullet(
    trigger: Trigger<SpawnBullet>,
    mut cmd: Commands,
    assets: ResMut<BulletAssets>,
    mut network_mapping: ResMut<NetworkMapping>,
) {
    let &SpawnBullet(entity, attacker, receiver, pos) = trigger.event();
    let Some(attacker) = network_mapping.get(&attacker) else {
        return;
    };
    let Some(receiver) = network_mapping.get(&receiver) else {
        return;
    };

    let bullet = cmd
        .spawn((
            Bullet,
            pos,
            AseSpriteSlice {
                name: "bullet".into(),
                aseprite: assets.sprite.clone(),
            },
            ProjectileTargets(*attacker, *receiver, 0.5),
        ))
        .id();

    network_mapping.insert(entity, bullet);
}
