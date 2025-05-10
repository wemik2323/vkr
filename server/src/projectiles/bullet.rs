use bevy::prelude::*;
use bevy_quinnet::server::QuinnetServer;
use common::{ArenaPos, Health, PlayerNumber, Projectile, ServerChannel, ServerMessage};

use crate::{ai::Movement, units::Hitbox};

use super::ProjectileRadius;

pub(super) fn plugin(app: &mut App) {
    app.add_observer(spawn_bullet);

    app.add_systems(FixedUpdate, update_bullets);
}

#[derive(Event)]
pub struct SpawnBullet(pub Entity, pub Entity, pub ArenaPos);

#[derive(Component)]
#[require(
    Projectile(|| Projectile::Bullet),
    ProjectileRadius(|| ProjectileRadius(0.2)),
)]
struct Bullet(Entity);

fn spawn_bullet(
    trigger: Trigger<SpawnBullet>,
    mut server: ResMut<QuinnetServer>,
    mut cmd: Commands,
) {
    let &SpawnBullet(attacker, receiver, pos) = trigger.event();

    let entity = cmd
        .spawn((
            Bullet(receiver),
            pos,
            Movement {
                target: Some(receiver),
                speed: 40.,
            },
        ))
        .id();

    server
        .endpoint_mut()
        .broadcast_message_on(
            ServerChannel::OrderedReliable,
            ServerMessage::SpawnProjectile {
                server_entity: entity,
                projectile: Projectile::Bullet,
                attacker,
                receiver,
                pos,
            },
        )
        .unwrap();
}

fn update_bullets(
    mut bullets: Query<
        (Entity, &Bullet, &ProjectileRadius, &mut ArenaPos),
        Without<PlayerNumber>,
    >,
    mut units: Query<(&ArenaPos, &mut Health, &Hitbox), With<PlayerNumber>>,
    mut cmd: Commands,
    mut server: ResMut<QuinnetServer>,
) {
    for (entity, bullet, radius, pos) in &mut bullets {
        let Ok((recv_pos, mut recv_health, hitbox)) = units.get_mut(bullet.0) else {
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

        recv_health.0 = recv_health.0.saturating_sub(50);
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
