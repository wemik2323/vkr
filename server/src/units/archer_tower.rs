use bevy::prelude::*;
use bevy_quinnet::server::QuinnetServer;
use common::{
    ArenaPos, Health, PlayerNumber, Projectile, ServerChannel, ServerMessage, Unit, UnitState,
};

use crate::ai::{Attack, AttackTargetType, AttackType};

use super::{Hitbox, UnitType};

pub(super) fn plugin(app: &mut App) {
    app.add_observer(spawn_archer_tower);
}

#[derive(Event)]
pub struct SpawnArcherTower(pub ArenaPos, pub PlayerNumber);

#[derive(Component)]
#[require(
    Health(|| Health::new(1400)),
    UnitType(|| UnitType::Ground),
    UnitState,
    Attack(|| Attack::new(AttackType::Ranged(Projectile::Bullet),
        AttackTargetType::All, 0.75, 8.5)),
    Hitbox(|| Hitbox(1.5)),
)]
struct ArcherTower;

fn spawn_archer_tower(
    trigger: Trigger<SpawnArcherTower>,
    mut server: ResMut<QuinnetServer>,
    mut cmd: Commands,
) {
    let &SpawnArcherTower(pos, owner) = trigger.event();

    let entity = cmd.spawn((ArcherTower, pos, owner)).id();

    server
        .endpoint_mut()
        .broadcast_message_on(
            ServerChannel::OrderedReliable,
            ServerMessage::SpawnUnit {
                server_entity: entity,
                unit: Unit::ArcherTower,
                pos,
                owner,
            },
        )
        .unwrap();
}
