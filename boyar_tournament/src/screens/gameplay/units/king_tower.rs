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

use super::{AssociatedTower, IntoTag, SpawnDirection};

pub(super) fn plugin(app: &mut App) {
    app.add_observer(spawn_king_tower);

    app.configure_loading_state(
        LoadingStateConfig::new(GameState::Loading).load_collection::<KingTowerAssets>(),
    );

    app.add_systems(OnExit(GameState::Gameplay), despawn_king_towers);
}

#[derive(Event)]
pub struct SpawnKingTower(pub Entity, pub ArenaPos, pub PlayerNumber);

#[derive(Component)]
#[require(
    DynamicScale(|| DynamicScale(0.75)),
    ArenaHeightOffset(|| ArenaHeightOffset(1.3)),
)]
struct KingTower;

#[derive(Component)]
#[require(
    Health,

    Name(|| Name::new("Король на башне")),
    DynamicScale(|| DynamicScale(0.55)),
    UnitState,
    ArenaHeightOffset(|| ArenaHeightOffset(3.)),
)]
struct KingTowerKing;

#[derive(Resource, AssetCollection)]
struct KingTowerAssets {
    #[asset(path = "units/king_tower/ally_tower.aseprite")]
    ally_tower: Handle<Aseprite>,
    #[asset(path = "units/king_tower/enemy_tower.aseprite")]
    enemy_tower: Handle<Aseprite>,

    #[asset(path = "units/priest/ally_priest.aseprite")]
    ally_king: Handle<Aseprite>,
    #[asset(path = "units/priest/enemy_priest.aseprite")]
    enemy_king: Handle<Aseprite>,
}

fn spawn_king_tower(
    trigger: Trigger<SpawnKingTower>,
    mut cmd: Commands,
    self_num: Res<PlayerNumber>,
    assets: ResMut<KingTowerAssets>,
    mut network_mapping: ResMut<NetworkMapping>,
) {
    let SpawnKingTower(entity, mut pos, player_num) = trigger.event();

    let direction = self_num.spawn_direction(*player_num);

    let (tower_sprite, king_sprite) = if pos.1 < 0. {
        (assets.ally_tower.clone(), assets.ally_king.clone())
    } else {
        (assets.enemy_tower.clone(), assets.enemy_king.clone())
    };

    pos.1 += 0.01;
    let tower = cmd
        .spawn((
            KingTower,
            pos,
            AseSpriteSlice {
                name: "king_tower".into(),
                aseprite: tower_sprite,
            },
        ))
        .id();

    pos.1 -= 0.01;
    let king = cmd
        .spawn((
            KingTowerKing,
            pos,
            direction,
            AseSpriteAnimation {
                animation: Animation::tag(direction.tag()),
                aseprite: king_sprite,
            },
            AssociatedTower(tower),
        ))
        .id();
    network_mapping.insert(*entity, king);
}

fn despawn_king_towers(mut cmd: Commands, towers: Query<(Entity, &AssociatedTower)>) {
    for (king, tower) in towers.iter() {
        cmd.entity(tower.0).despawn();
        cmd.entity(king).despawn();
    }
}
