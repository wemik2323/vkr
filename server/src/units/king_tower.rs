use bevy::prelude::*;
use bevy_quinnet::server::QuinnetServer;
use common::{
    ArenaPos, Health, PlayerNumber, Projectile, ServerChannel, ServerMessage, Unit, UnitState,
};

use crate::ai::{Attack, AttackTargetType, AttackType};

use super::{Hitbox, UnitType};

pub(super) fn plugin(app: &mut App) {
    app.add_observer(spawn_king_tower);
}

#[derive(Event)]
pub struct SpawnKingTower(pub ArenaPos, pub PlayerNumber);

#[derive(Component)]
#[require(
    Health(|| Health::new(2400)),
    UnitType(|| UnitType::Ground),
    UnitState,
    Attack(|| Attack::new(AttackType::Ranged(Projectile::Fireball),
        AttackTargetType::All, 1., 6.)),
    Hitbox(|| Hitbox(2.)),
)]
struct KingTower;

fn spawn_king_tower(
    trigger: Trigger<SpawnKingTower>,
    mut server: ResMut<QuinnetServer>,
    mut cmd: Commands,
) {
    let &SpawnKingTower(pos, owner) = trigger.event();

    let entity = cmd.spawn((KingTower, pos, owner)).id();

    server
        .endpoint_mut()
        .broadcast_message_on(
            ServerChannel::OrderedReliable,
            ServerMessage::SpawnUnit {
                server_entity: entity,
                unit: Unit::KingTower,
                pos,
                owner,
            },
        )
        .unwrap();
}
