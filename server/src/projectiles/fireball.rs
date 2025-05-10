use bevy::prelude::*;
use bevy_quinnet::server::QuinnetServer;
use common::{ArenaPos, Health, PlayerNumber, Projectile, ServerChannel, ServerMessage};

use crate::{ai::Movement, units::Hitbox};

use super::ProjectileRadius;

pub(super) fn plugin(app: &mut App) {
    app.add_observer(spawn_fireball);

    app.add_systems(FixedUpdate, update_fireballs);
}

#[derive(Event)]
pub struct SpawnFireball(pub Entity, pub Entity, pub ArenaPos);

#[derive(Component)]
#[require(
    Projectile(|| Projectile::Fireball),
    ProjectileRadius(|| ProjectileRadius(1.)),
)]
struct Fireball(Entity);

fn spawn_fireball(
    trigger: Trigger<SpawnFireball>,
    mut server: ResMut<QuinnetServer>,
    mut cmd: Commands,
) {
    let &SpawnFireball(attacker, receiver, pos) = trigger.event();

    let entity = cmd
        .spawn((
            Fireball(receiver),
            pos,
            Movement {
                target: Some(receiver),
                speed: 10.,
            },
        ))
        .id();

    server
        .endpoint_mut()
        .broadcast_message_on(
            ServerChannel::OrderedReliable,
            ServerMessage::SpawnProjectile {
                server_entity: entity,
                projectile: Projectile::Fireball,
                attacker,
                receiver,
                pos,
            },
        )
        .unwrap();
}

fn update_fireballs(
    mut fireballs: Query<
        (Entity, &Fireball, &ProjectileRadius, &mut ArenaPos),
        Without<PlayerNumber>,
    >,
    mut units: Query<(&ArenaPos, &mut Health, &Hitbox), With<PlayerNumber>>,
    mut cmd: Commands,
    mut server: ResMut<QuinnetServer>,
) {
    for (entity, fireball, radius, pos) in &mut fireballs {
        let Ok((recv_pos, _, hitbox)) = units.get_mut(fireball.0) else {
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

        for (recv_pos, mut recv_health, hitbox) in &mut units {
            if pos.distance(recv_pos) > radius.0 + hitbox.0 {
                continue;
            }
            recv_health.0 = recv_health.0.saturating_sub(140);
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
