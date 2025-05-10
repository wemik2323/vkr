use bevy::prelude::*;
use bevy_quinnet::server::QuinnetServer;
use common::{
    ArenaPos, Health, PlayerNumber, Projectile, ServerChannel, ServerMessage, UnitState,
};

use crate::{projectiles::SpawnProjectile, units::UnitType};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        (
            (
                update_attacks,
                update_unit_state,
                update_stun_timers,
                update_movement,
            ),
            check_health,
        )
            .chain(),
    );
}

pub enum AttackType {
    Melee(u16), // Урон
    Ranged(Projectile),
}

pub enum AttackTargetType {
    Ground,
    All,
}

#[derive(Component)]
pub struct Attack {
    pub target: Option<Entity>,
    pub a_type: AttackType,
    pub t_type: AttackTargetType,
    pub cooldown_timer: Timer,
    pub range: f32,
}
impl Attack {
    pub fn new(a_type: AttackType, targets: AttackTargetType, cd: f32, range: f32) -> Self {
        Self {
            target: None,
            a_type,
            t_type: targets,
            cooldown_timer: Timer::from_seconds(cd, TimerMode::Repeating),
            range,
        }
    }
}

fn update_attacks(
    mut attacks: Query<(Entity, &mut Attack)>,
    mut units: Query<(&ArenaPos, &mut Health)>,
    time: Res<Time>,
    mut cmd: Commands,
) {
    for (attacker, mut attack) in &mut attacks {
        // target есть только в UnitState::Attacking
        let Some(receiver) = attack.target else {
            attack.cooldown_timer.reset();
            continue;
        };
        let Ok((_, mut health)) = units.get_mut(receiver) else {
            // Все мертвы
            attack.target = None;
            continue;
        };
        if !attack.cooldown_timer.tick(time.delta()).just_finished() {
            continue;
        }

        match attack.a_type {
            AttackType::Melee(damage) => health.0 = health.0.saturating_sub(damage),
            AttackType::Ranged(projectile) => {
                let (pos, _) = units.get(attacker).unwrap();
                projectile.spawn(attacker, receiver, *pos, &mut cmd)
            }
        }
    }
}

fn check_health(
    query: Query<(Entity, &Health)>,
    mut server: ResMut<QuinnetServer>,
    mut cmd: Commands,
) {
    for (entity, health) in &query {
        if health.0 == 0 {
            cmd.entity(entity).despawn();
            server
                .endpoint_mut()
                .broadcast_message_on(
                    ServerChannel::OrderedReliable,
                    ServerMessage::Despawn(entity),
                )
                .unwrap();
        }
    }
}

#[derive(Component)]
pub struct Movement {
    pub target: Option<Entity>,
    pub speed: f32,
}
impl Movement {
    pub fn new(speed: f32) -> Self {
        Self {
            target: None,
            speed,
        }
    }
}
fn update_movement(
    mut query: Query<(Entity, &mut Movement), Without<StunnedTimer>>,
    states: Query<&UnitState>,
    mut positions: Query<&mut ArenaPos>,
    time: Res<Time>,
) {
    for (entity, mut movement) in &mut query {
        if let Ok(state) = states.get(entity) {
            let UnitState::Moving = state else {
                continue;
            };
        }
        // target устанавливается в update_unit_state
        let Some(target) = movement.target else {
            continue;
        };
        let Ok(&target_pos) = positions.get(target) else {
            movement.target = None;
            continue;
        };
        let Ok(mut self_pos) = positions.get_mut(entity) else {
            continue;
        };
        let direction = self_pos.direction(&target_pos);
        *self_pos += direction.mul(movement.speed * time.delta_secs());
    }
}

#[derive(Component)]
// Таймер добавляется при спавне юнитов
pub struct StunnedTimer(pub Timer);
impl Default for StunnedTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(1.5, TimerMode::Once))
    }
}
fn update_stun_timers(
    mut query: Query<(Entity, &mut StunnedTimer)>,
    mut cmd: Commands,
    time: Res<Time>,
) {
    for (entity, mut timer) in &mut query {
        if timer.0.tick(time.delta()).just_finished() {
            cmd.entity(entity).remove::<StunnedTimer>();
        }
    }
}

#[derive(Component)]
pub struct AggroRadius(pub f32);

fn update_unit_state(
    mut attackers: Query<
        (
            Entity,
            &mut UnitState,
            &mut Attack,
            Option<&AggroRadius>,
            Option<&mut Movement>,
        ),
        Without<StunnedTimer>,
    >,
    receivers: Query<(Entity, &ArenaPos, &PlayerNumber, &UnitType)>,
    towers: Query<(Entity, &ArenaPos, &PlayerNumber), Without<Movement>>,
) {
    'outer: for (self_entity, mut state, mut attack, aggro_radius, mut movement) in
        &mut attackers
    {
        match *state {
            UnitState::Idle | UnitState::Moving => {
                let (_, self_pos, self_player_numer, _) = receivers.get(self_entity).unwrap();

                for (entity, pos, player_number, unit_type) in &receivers {
                    if self_player_numer == player_number {
                        // Своих не бьём
                        continue;
                    }
                    if let (AttackTargetType::Ground, UnitType::Air) =
                        (&attack.t_type, unit_type)
                    {
                        continue;
                    }

                    if self_pos.distance(pos) <= attack.range {
                        *state = UnitState::Attacking;
                        attack.target = Some(entity);
                        continue 'outer;
                    }

                    // У всего что не является постройкой есть и AggroRadius и Movement
                    if let (Some(aggro_radius), Some(movement)) =
                        (aggro_radius, movement.as_mut())
                    {
                        if self_pos.distance(pos) <= aggro_radius.0 {
                            movement.target = Some(entity);
                            continue 'outer;
                        }
                    }
                }

                // Если никого нет вблизи, двигаемся к ближайшей башне
                let Some(movement) = movement.as_mut() else {
                    continue;
                };
                let mut nearest_tower = None;
                let mut minimal_distance = 1000.;
                for (tower_entity, tower_pos, tower_player_number) in &towers {
                    let distance = self_pos.distance(tower_pos);
                    if self_player_numer == tower_player_number || distance > minimal_distance
                    {
                        continue;
                    }

                    minimal_distance = distance;
                    nearest_tower = Some(tower_entity);
                }
                let Some(nearest_tower) = nearest_tower else {
                    // Башен врага не осталось, игра должна закончиться
                    continue;
                };
                movement.target = Some(nearest_tower);
            }
            UnitState::Attacking => {
                if let Some(target) = attack.target {
                    let (_, self_pos, _, _) = receivers.get(self_entity).unwrap();

                    if let Ok((_, pos, _, _)) = receivers.get(target) {
                        if self_pos.distance(pos) > attack.range {
                            match movement.as_mut() {
                                Some(_) => *state = UnitState::Moving,
                                None => *state = UnitState::Idle,
                            }
                        }
                        continue;
                    };

                    continue;
                }

                match movement.as_mut() {
                    Some(_) => *state = UnitState::Moving,
                    None => *state = UnitState::Idle,
                }
            }
        }
    }
}
