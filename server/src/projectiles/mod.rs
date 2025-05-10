use bevy::prelude::*;
use bomb::SpawnBomb;
use bullet::SpawnBullet;
use common::{ArenaPos, Projectile};
use fireball::SpawnFireball;

mod bomb;
mod bullet;
mod fireball;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((bullet::plugin, fireball::plugin, bomb::plugin));
}

#[derive(Component)]
struct ProjectileRadius(pub f32);

pub(super) trait SpawnProjectile {
    fn spawn(&self, attacker: Entity, receiver: Entity, pos: ArenaPos, cmd: &mut Commands);
}

impl SpawnProjectile for Projectile {
    fn spawn(&self, attacker: Entity, receiver: Entity, pos: ArenaPos, cmd: &mut Commands) {
        match self {
            Projectile::Bullet => cmd.trigger(SpawnBullet(attacker, receiver, pos)),
            Projectile::Fireball => cmd.trigger(SpawnFireball(attacker, receiver, pos)),
            Projectile::Bomb => cmd.trigger(SpawnBomb(attacker, receiver, pos)),
        }
    }
}
