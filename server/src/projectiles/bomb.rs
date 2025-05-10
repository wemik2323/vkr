use bevy::prelude::*;
use bevy_quinnet::server::QuinnetServer;
use common::{ArenaPos, Health, PlayerNumber, Projectile, ServerChannel, ServerMessage};

use crate::{ai::Movement, units::{Hitbox, UnitType}};

use super::ProjectileRadius;

pub(super) fn plugin(app: &mut App) {
    app.add_observer(spawn_bomb);

    app.add_systems(FixedUpdate, update_bombs);
}

#[derive(Event)]
pub struct SpawnBomb(pub Entity, pub Entity, pub ArenaPos);

#[derive(Component)]
#[require(
    Projectile(|| Projectile::Bomb),
    ProjectileRadius(|| ProjectileRadius(1.)),
)]
struct Bomb(Entity);

fn spawn_bomb(
    trigger: Trigger<SpawnBomb>,
    mut server: ResMut<QuinnetServer>,
    mut cmd: Commands,
) {
    let &SpawnBomb(attacker, receiver, pos) = trigger.event();

    let entity = cmd
        .spawn((
            Bomb(receiver),
            pos,
            Movement {
                target: Some(receiver),
                speed: 15.,
            },
        ))
        .id();

    server
        .endpoint_mut()
        .broadcast_message_on(
            ServerChannel::OrderedReliable,
            ServerMessage::SpawnProjectile {
                server_entity: entity,
                projectile: Projectile::Bomb,
                attacker,
                receiver,
                pos,
            },
        )
        .unwrap();
}

fn update_bombs(
    mut bombs: Query<
        (Entity, &Bomb, &ProjectileRadius, &mut ArenaPos),
        Without<PlayerNumber>,
    >,
    mut units: Query<(&ArenaPos, &mut Health, &Hitbox, &UnitType), With<PlayerNumber>>,
    mut cmd: Commands,
    mut server: ResMut<QuinnetServer>,
) {
    for (entity, bomb, radius, pos) in &mut bombs {
        let Ok((recv_pos, _, hitbox, _)) = units.get_mut(bomb.0) else {
            // Цель умерла
            cmd.entity(entity).despawn();
            server
                .endpoint_mut()
                .broadcast_message_on(
                    ServerChannel::OrderedReliable,
                    ServerMessage::Despawn(entity),
                )
                .unwrap();
            continue;
        };

        if pos.distance(recv_pos) > radius.0 + hitbox.0 {
            continue;
        }

        for (recv_pos, mut recv_health, hitbox, unit_type) in &mut units {
            if let UnitType::Air = unit_type {
                continue;
            }
            if pos.distance(recv_pos) > radius.0 + hitbox.0 {
                continue;
            }
            recv_health.0 = recv_health.0.saturating_sub(88);
        }
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
