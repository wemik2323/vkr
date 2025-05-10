use bevy::prelude::*;
use bevy_quinnet::server::QuinnetServer;
use common::{
    ArenaPos, Health, PlayerNumber, Projectile, ServerChannel, ServerMessage, Unit, UnitState,
};

use crate::ai::{AggroRadius, Attack, AttackTargetType, AttackType, Movement, StunnedTimer};

use super::{Hitbox, UnitType};

pub(super) fn plugin(app: &mut App) {
    app.add_observer(spawn_bomber);
}

#[derive(Event)]
pub struct SpawnBomber(pub ArenaPos, pub PlayerNumber);

#[derive(Component)]
#[require(
    Health(|| Health::new(230)),
    Movement(|| Movement::new(2.)),
    AggroRadius(|| AggroRadius(5.5)),
    UnitType(|| UnitType::Ground),
    UnitState(|| UnitState::Moving),
    Attack(|| Attack::new(AttackType::Ranged(Projectile::Bomb),
        AttackTargetType::Ground, 0.7, 4.5)),
    Hitbox(|| Hitbox(0.5)),
    StunnedTimer,
)]
struct Bomber;

fn spawn_bomber(
    trigger: Trigger<SpawnBomber>,
    mut server: ResMut<QuinnetServer>,
    mut cmd: Commands,
) {
    let &SpawnBomber(pos, owner) = trigger.event();

    let entity = cmd.spawn((Bomber, pos, owner)).id();

    server
        .endpoint_mut()
        .broadcast_message_on(
            ServerChannel::OrderedReliable,
            ServerMessage::SpawnUnit {
                server_entity: entity,
                unit: Unit::Bomber,
                pos,
                owner,
            },
        )
        .unwrap();
}
