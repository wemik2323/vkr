use bevy::prelude::*;
use bevy_quinnet::server::QuinnetServer;
use common::{ArenaPos, Health, PlayerNumber, ServerChannel, ServerMessage, Unit, UnitState};

use crate::ai::{Movement, StunnedTimer};

use super::{Hitbox, UnitType};

pub(super) fn plugin(app: &mut App) {
    app.add_observer(spawn_giant);

    app.add_systems(FixedUpdate, update_giants);
}

#[derive(Event)]
pub struct SpawnGiant(pub ArenaPos, pub PlayerNumber);

#[derive(Component)]
#[require(
    Health(|| Health::new(800)),
    Movement(|| Movement::new(1.5)),
    UnitType(|| UnitType::Ground),
    UnitState(|| UnitState::Moving),
    Hitbox(|| Hitbox(1.)),
    StunnedTimer,
)]
pub struct Giant {
    pub target: Option<Entity>,
    pub attack_range: f32,
    pub damage: u16,
    pub cooldown: Timer,
}

fn spawn_giant(
    trigger: Trigger<SpawnGiant>,
    mut server: ResMut<QuinnetServer>,
    mut cmd: Commands,
) {
    let &SpawnGiant(pos, owner) = trigger.event();

    let entity = cmd
        .spawn((
            Giant {
                target: None,
                attack_range: 2.,
                damage: 120,
                cooldown: Timer::from_seconds(1.5, TimerMode::Repeating),
            },
            pos,
            owner,
        ))
        .id();

    server
        .endpoint_mut()
        .broadcast_message_on(
            ServerChannel::OrderedReliable,
            ServerMessage::SpawnUnit {
                server_entity: entity,
                unit: Unit::Giant,
                pos,
                owner,
            },
        )
        .unwrap();
}

fn update_giants(
    mut giants: Query<(
        &mut Giant,
        &mut UnitState,
        &mut Movement,
        &ArenaPos,
        &PlayerNumber,
    )>,
    mut towers: Query<(Entity, &ArenaPos, &mut Health, &PlayerNumber), Without<Movement>>,
    time: Res<Time>,
) {
    for (mut giant, mut state, mut movement, pos, player_num) in &mut giants {
        match *state {
            UnitState::Idle => panic!("Гигант не может находиться в UnitState::Idle"),
            UnitState::Moving => {
                if let Some(target) = movement.target {
                    let Ok((tower, tower_pos, _, _)) = towers.get(target) else {
                        continue;
                    };

                    if pos.distance(tower_pos) <= giant.attack_range {
                        *state = UnitState::Attacking;
                        giant.target = Some(tower);
                    }
                    continue;
                };

                let mut closest_tower = None;
                let mut minimal_distance = 1000.;
                for (tower, tower_pos, _, tower_player_num) in &towers {
                    if player_num == tower_player_num {
                        continue;
                    }
                    let distance = pos.distance(tower_pos);
                    if distance < minimal_distance {
                        closest_tower = Some(tower);
                        minimal_distance = distance;
                    }
                }
                movement.target = closest_tower;
            }
            UnitState::Attacking => {
                if let Some(target) = giant.target {
                    if !giant.cooldown.tick(time.delta()).just_finished() {
                        continue;
                    }

                    let Ok((_, _, mut health, _)) = towers.get_mut(target) else {
                        giant.target = None;
                        *state = UnitState::Moving;
                        continue;
                    };
                    health.0 = health.0.saturating_sub(giant.damage);
                    continue;
                }
            }
        }
    }
}
