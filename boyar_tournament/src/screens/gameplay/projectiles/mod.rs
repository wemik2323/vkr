use bevy::prelude::*;
use bomb::SpawnBomb;
use bullet::SpawnBullet;
use common::{ArenaPos, Projectile};
use fireball::SpawnFireball;

use crate::screens::GameState;

use super::arena::ArenaHeightOffset;

mod bomb;
mod bullet;
mod fireball;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((bullet::plugin, fireball::plugin, bomb::plugin));

    app.add_systems(
        Update,
        update_projectile_height.run_if(in_state(GameState::Gameplay)),
    );
}

pub(super) trait SpawnProjectile {
    fn spawn(
        &self,
        entity: Entity,
        attacker: Entity,
        receiver: Entity,
        pos: ArenaPos,
        cmd: &mut Commands,
    );
}

impl SpawnProjectile for Projectile {
    fn spawn(
        &self,
        entity: Entity,
        attacker: Entity,
        receiver: Entity,
        pos: ArenaPos,
        cmd: &mut Commands,
    ) {
        match self {
            Projectile::Bullet => cmd.trigger(SpawnBullet(entity, attacker, receiver, pos)),
            Projectile::Fireball => {
                cmd.trigger(SpawnFireball(entity, attacker, receiver, pos))
            }
            Projectile::Bomb => cmd.trigger(SpawnBomb(entity, attacker, receiver, pos)),
        }
    }
}

#[derive(Component)]
struct ProjectileTargets(Entity, Entity, f32);

fn update_projectile_height(
    projectiles: Query<(Entity, &ProjectileTargets)>,
    mut positions: Query<(&ArenaPos, &mut ArenaHeightOffset)>,
) {
    for (entity, targets) in &projectiles {
        let Ok((&attacker_pos, &attacker_height)) = positions.get(targets.0) else {
            continue;
        };
        let Ok((&receiver_pos, &receiver_height)) = positions.get(targets.1) else {
            continue;
        };
        let Ok((self_pos, mut self_height)) = positions.get_mut(entity) else {
            continue;
        };

        let dist_to_attacker = self_pos.distance(&attacker_pos);
        let dist_to_receiver = self_pos.distance(&receiver_pos);
        let progress = dist_to_attacker / (dist_to_attacker + dist_to_receiver);

        let height = attacker_height.0 + progress * (receiver_height.0 - attacker_height.0);
        self_height.0 = height + targets.2;
    }
}
