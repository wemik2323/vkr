use bevy::prelude::*;
use bevy_quinnet::server::QuinnetServer;
use common::{ArenaPos, Health, PlayerNumber, ServerChannel, ServerMessage, Unit, UnitState};

use crate::ai::{AggroRadius, Attack, AttackTargetType, AttackType, Movement, StunnedTimer};

use super::{Hitbox, UnitType};

pub(super) fn plugin(app: &mut App) {
    app.add_observer(spawn_bat);
}

#[derive(Event)]
pub struct SpawnBat(pub ArenaPos, pub PlayerNumber);

#[derive(Component)]
#[require(
    Health(|| Health::new(90)),
    Movement(|| Movement::new(3.)),
    AggroRadius(|| AggroRadius(5.)),
    UnitType(|| UnitType::Air),
    UnitState(|| UnitState::Moving),
    Attack(|| Attack::new(AttackType::Melee(80), AttackTargetType::All, 1., 2.)),
    Hitbox(|| Hitbox(0.5)),
    StunnedTimer,
)]
struct Bat;

fn spawn_bat(
    trigger: Trigger<SpawnBat>,
    mut server: ResMut<QuinnetServer>,
    mut cmd: Commands,
) {
    let &SpawnBat(pos, owner) = trigger.event();

    let entity = cmd.spawn((Bat, pos, owner)).id();

    server
        .endpoint_mut()
        .broadcast_message_on(
            ServerChannel::OrderedReliable,
            ServerMessage::SpawnUnit {
                server_entity: entity,
                unit: Unit::Bat,
                pos,
                owner,
            },
        )
        .unwrap();
}
