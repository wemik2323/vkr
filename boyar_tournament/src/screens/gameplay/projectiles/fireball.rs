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
    app.add_observer(spawn_fireball);

    app.configure_loading_state(
        LoadingStateConfig::new(GameState::Loading).load_collection::<FireballAssets>(),
    );
}

#[derive(Event)]
pub struct SpawnFireball(pub Entity, pub Entity, pub Entity, pub ArenaPos);

#[derive(Component)]
#[require(
    Name(|| Name::new("Фаерболл")),
    DynamicScale(|| DynamicScale(0.5)),
    ArenaHeightOffset(|| ArenaHeightOffset(0.)),
)]
struct Fireball;

#[derive(Resource, AssetCollection)]
struct FireballAssets {
    #[asset(path = "units/priest/fireball.aseprite")]
    sprite: Handle<Aseprite>,
}

fn spawn_fireball(
    trigger: Trigger<SpawnFireball>,
    mut cmd: Commands,
    assets: ResMut<FireballAssets>,
    mut network_mapping: ResMut<NetworkMapping>,
) {
    let &SpawnFireball(entity, attacker, receiver, pos) = trigger.event();
    let Some(attacker) = network_mapping.get(&attacker) else {
        return;
    };
    let Some(receiver) = network_mapping.get(&receiver) else {
        return;
    };

    let fireball = cmd
        .spawn((
            Fireball,
            pos,
            AseSpriteAnimation {
                animation: Animation::tag("fireball"),
                aseprite: assets.sprite.clone(),
            },
            ProjectileTargets(*attacker, *receiver, 0.5),
        ))
        .id();

    network_mapping.insert(entity, fireball);
}
