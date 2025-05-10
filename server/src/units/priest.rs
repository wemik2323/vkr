use bevy::prelude::*;
use bevy_quinnet::server::QuinnetServer;
use common::{
    ArenaPos, Health, PlayerNumber, Projectile, ServerChannel, ServerMessage, Unit, UnitState,
};

use crate::ai::{AggroRadius, Attack, AttackTargetType, AttackType, Movement, StunnedTimer};

use super::{Hitbox, UnitType};

pub(super) fn plugin(app: &mut App) {
    app.add_observer(spawn_priest);
}

#[derive(Event)]
pub struct SpawnPriest(pub ArenaPos, pub PlayerNumber);

#[derive(Component)]
#[require(
    Health(|| Health::new(400)),
    Movement(|| Movement::new(2.)),
    AggroRadius(|| AggroRadius(7.)),
    UnitType(|| UnitType::Ground),
    UnitState(|| UnitState::Moving),
    Attack(|| Attack::new(AttackType::Ranged(Projectile::Fireball),
        AttackTargetType::All, 0.75, 6.)),
    Hitbox(|| Hitbox(0.5)),
    StunnedTimer,
)]
struct Priest;

fn spawn_priest(
    trigger: Trigger<SpawnPriest>,
    mut server: ResMut<QuinnetServer>,
    mut cmd: Commands,
) {
    let &SpawnPriest(pos, owner) = trigger.event();

    let entity = cmd.spawn((Priest, pos, owner)).id();

    server
        .endpoint_mut()
        .broadcast_message_on(
            ServerChannel::OrderedReliable,
            ServerMessage::SpawnUnit {
                server_entity: entity,
                unit: Unit::Priest,
                pos,
                owner,
            },
        )
        .unwrap();
}
