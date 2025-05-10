use bevy::{prelude::*, utils::HashMap};
use bevy_quinnet::client::{
    certificate::CertificateVerificationMode, connection::ClientEndpointConfiguration,
    QuinnetClient, QuinnetClientPlugin,
};
use common::{
    ArenaPos, ClientChannel, Direction, Health, PlayerNumber, ServerMessage, UnitState,
    LOCAL_BIND_IP, SERVER_HOST, SERVER_PORT,
};

use crate::screens::GameState;

use super::{
    projectiles::SpawnProjectile,
    units::{AssociatedTower, SpawnUnit},
};

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(QuinnetClientPlugin::default());

    app.init_resource::<PlayerNumber>();
    app.init_resource::<NetworkMapping>();
    app.register_type::<NetworkMapping>();

    app.add_systems(OnEnter(GameState::Gameplay), start_connection);
    app.add_systems(
        Update,
        handle_server_messages.run_if(in_state(GameState::Gameplay)),
    );
}

fn start_connection(mut client: ResMut<QuinnetClient>) {
    client
        .open_connection(
            ClientEndpointConfiguration::from_ips(SERVER_HOST, SERVER_PORT, LOCAL_BIND_IP, 0),
            CertificateVerificationMode::SkipVerification,
            ClientChannel::channels_config(),
        )
        .unwrap();
}

trait AdjustForPlayer {
    fn adjust_for_player(&self, player_num: PlayerNumber) -> Self;
}
impl AdjustForPlayer for ArenaPos {
    fn adjust_for_player(&self, player_num: PlayerNumber) -> Self {
        match player_num {
            PlayerNumber::One => *self,
            PlayerNumber::Two => ArenaPos(-self.0, -self.1),
        }
    }
}
impl AdjustForPlayer for Direction {
    fn adjust_for_player(&self, player_num: PlayerNumber) -> Self {
        match player_num {
            PlayerNumber::One => *self,
            PlayerNumber::Two => self.opposite(),
        }
    }
}

fn handle_server_messages(
    mut client: ResMut<QuinnetClient>,
    mut player_num: ResMut<PlayerNumber>,
    mut cmd: Commands,
    mut network_mapping: ResMut<NetworkMapping>,
    mut units_query: Query<(&mut ArenaPos, &mut Direction, &mut UnitState, &mut Health)>,
    mut projectiles_query: Query<&mut ArenaPos, Without<UnitState>>,
    towers: Query<&AssociatedTower>,
) {
    while let Some((_, message)) = client
        .connection_mut()
        .try_receive_message::<ServerMessage>()
    {
        match message {
            ServerMessage::StartGame(n) => *player_num = n,
            ServerMessage::SpawnUnit {
                server_entity,
                unit,
                pos,
                owner,
            } => {
                unit.spawn(
                    server_entity,
                    pos.adjust_for_player(*player_num),
                    owner,
                    &mut cmd,
                );
            }
            ServerMessage::SpawnProjectile {
                server_entity,
                projectile,
                attacker,
                receiver,
                pos,
            } => projectile.spawn(
                server_entity,
                attacker,
                receiver,
                pos.adjust_for_player(*player_num),
                &mut cmd,
            ),
            ServerMessage::Despawn(server_entity) => {
                let Some(entity) = network_mapping.remove(&server_entity) else {
                    continue;
                };
                if let Ok(tower) = towers.get(entity) {
                    cmd.entity(tower.0).despawn();
                }
                cmd.entity(entity).despawn();
            }
            ServerMessage::SyncEntities { units, projectiles } => {
                for (server_entity, pos, direction, state, health) in &units {
                    let Some(&entity) = network_mapping.get(server_entity) else {
                        continue;
                    };
                    let (mut p, mut d, mut s, mut h) = units_query.get_mut(entity).unwrap();
                    *p = pos.adjust_for_player(*player_num);
                    *d = direction.adjust_for_player(*player_num);
                    *s = *state;
                    *h = *health;
                }

                for (server_entity, pos) in &projectiles {
                    let Some(&entity) = network_mapping.get(server_entity) else {
                        continue;
                    };
                    let mut p = projectiles_query.get_mut(entity).unwrap();
                    *p = pos.adjust_for_player(*player_num);
                }
            }
        }
    }
}

#[derive(Resource, Reflect, Default, Deref, DerefMut)]
#[reflect(Resource)]
// Сопоставление Entity сервера и клиента
pub struct NetworkMapping(HashMap<Entity, Entity>);
