use bevy::prelude::*;
use bevy_quinnet::server::QuinnetServer;
use common::{ArenaPos, Health, PlayerNumber, ServerChannel, ServerMessage, Unit, UnitState};

use crate::ai::{AggroRadius, Attack, AttackTargetType, AttackType, Movement, StunnedTimer};

use super::{Hitbox, UnitType};

pub(super) fn plugin(app: &mut App) {
    app.add_observer(spawn_rus);
}

#[derive(Event)]
pub struct SpawnRus(pub ArenaPos, pub PlayerNumber);

#[derive(Component)]
#[require(
    Health(|| Health::new(690)),
    Movement(|| Movement::new(2.)),
    AggroRadius(|| AggroRadius(5.)),
    UnitType(|| UnitType::Ground),
    UnitState(|| UnitState::Moving),
    Attack(|| Attack::new(AttackType::Melee(80), AttackTargetType::Ground, 0.8, 2.)),
    Hitbox(|| Hitbox(0.5)),
    StunnedTimer,
)]
struct Rus;

fn spawn_rus(
    trigger: Trigger<SpawnRus>,
    mut server: ResMut<QuinnetServer>,
    mut cmd: Commands,
) {
    let &SpawnRus(pos, owner) = trigger.event();

    let entity = cmd.spawn((Rus, pos, owner)).id();

    server
        .endpoint_mut()
        .broadcast_message_on(
            ServerChannel::OrderedReliable,
            ServerMessage::SpawnUnit {
                server_entity: entity,
                unit: Unit::Rus,
                pos,
                owner,
            },
        )
        .unwrap();
}
