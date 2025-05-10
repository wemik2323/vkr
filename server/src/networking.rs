use core::f32;

use bevy::{prelude::*, utils::HashMap};
use bevy_quinnet::{
    server::{
        certificate::CertificateRetrievalMode, ConnectionEvent, QuinnetServer,
        QuinnetServerPlugin, ServerEndpointConfiguration,
    },
    shared::ClientId,
};
use common::{
    ArenaPos, Card, ClientMessage, Direction, Health, PlayerNumber, ServerChannel,
    ServerMessage, Unit, UnitState, LOCAL_BIND_IP, SERVER_HOST, SERVER_PORT,
};

use crate::{
    ai::{Attack, Movement, StunnedTimer},
    units::{Giant, SpawnUnit},
};

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(QuinnetServerPlugin::default());

    app.init_resource::<Lobby>();
    app.add_systems(Startup, start_listening);
    app.add_systems(Update, (handle_connection_events, handle_client_messages));

    app.add_systems(FixedPostUpdate, sync_entities);
}

fn start_listening(mut server: ResMut<QuinnetServer>) {
    server
        .start_endpoint(
            ServerEndpointConfiguration::from_ip(LOCAL_BIND_IP, SERVER_PORT),
            CertificateRetrievalMode::GenerateSelfSigned {
                server_hostname: SERVER_HOST.to_string(),
            },
            ServerChannel::channels_config(),
        )
        .unwrap();
}

#[derive(Resource, Default, Deref, DerefMut)]
pub struct Lobby(HashMap<ClientId, PlayerNumber>);

fn handle_connection_events(
    mut connection_events: EventReader<ConnectionEvent>,
    mut lobby: ResMut<Lobby>,
    mut server: ResMut<QuinnetServer>,
    mut cmd: Commands,
) {
    let lobby_len = lobby.len() as u8;
    for client in connection_events.read() {
        if lobby_len >= 2 {
            server.endpoint_mut().disconnect_client(client.id).unwrap();
            continue;
        }
        use PlayerNumber::*;

        let player_num = match lobby_len {
            0 => One,
            1 => Two,
            _ => unreachable!(),
        };
        lobby.insert(client.id, player_num);

        if lobby.len() == 2 {
            // Отправить каждому игроку его PlayerNumber
            for (client_id, player_num) in lobby.iter() {
                server
                    .endpoint_mut()
                    .send_message_on(
                        *client_id,
                        ServerChannel::OrderedReliable,
                        ServerMessage::StartGame(*player_num),
                    )
                    .unwrap();
            }

            Unit::ArcherTower.spawn(ArenaPos(-5.5, -9.5), One, &mut cmd);
            Unit::KingTower.spawn(ArenaPos(0., -13.), One, &mut cmd);
            Unit::ArcherTower.spawn(ArenaPos(5.5, -9.5), One, &mut cmd);

            Unit::ArcherTower.spawn(ArenaPos(-5.5, 9.5), Two, &mut cmd);
            Unit::KingTower.spawn(ArenaPos(0., 13.), Two, &mut cmd);
            Unit::ArcherTower.spawn(ArenaPos(5.5, 9.5), Two, &mut cmd);
        }
    }
}

fn handle_client_messages(
    mut server: ResMut<QuinnetServer>,
    lobby: Res<Lobby>,
    mut cmd: Commands,
) {
    let endpoint = server.endpoint_mut();
    for client_id in endpoint.clients() {
        while let Some((_, message)) =
            endpoint.try_receive_message_from::<ClientMessage>(client_id)
        {
            let player_num = lobby.get(&client_id).unwrap();
            match message {
                ClientMessage::PlayCard { card, placement } => match card {
                    Card::Rus => Unit::Rus.spawn(placement, *player_num, &mut cmd),
                    Card::Musketeer => Unit::Musketeer.spawn(placement, *player_num, &mut cmd),
                    Card::ThreeMusketeers => {
                        let ArenaPos(x, y) = placement;
                        Unit::Musketeer.spawn(ArenaPos(x, y + 0.8), *player_num, &mut cmd);
                        Unit::Musketeer.spawn(ArenaPos(x + 0.8, y), *player_num, &mut cmd);
                        Unit::Musketeer.spawn(ArenaPos(x - 0.8, y), *player_num, &mut cmd);
                    }
                    Card::Bats => {
                        let ArenaPos(x, y) = placement;
                        Unit::Bat.spawn(ArenaPos(x, y + 0.8), *player_num, &mut cmd);
                        Unit::Bat.spawn(ArenaPos(x + 0.8, y), *player_num, &mut cmd);
                        Unit::Bat.spawn(ArenaPos(x - 0.8, y), *player_num, &mut cmd);
                    }
                    Card::BatHorde => {
                        let ArenaPos(x, y) = placement;
                        Unit::Bat.spawn(ArenaPos(x + 0.5, y + 0.5), *player_num, &mut cmd);
                        Unit::Bat.spawn(ArenaPos(x + 0.8, y), *player_num, &mut cmd);
                        Unit::Bat.spawn(ArenaPos(x + 0.5, y - 0.5), *player_num, &mut cmd);
                        Unit::Bat.spawn(ArenaPos(x - 0.5, y - 0.5), *player_num, &mut cmd);
                        Unit::Bat.spawn(ArenaPos(x - 0.8, y), *player_num, &mut cmd);
                        Unit::Bat.spawn(ArenaPos(x - 0.5, y + 0.5), *player_num, &mut cmd);
                    }
                    Card::Priest => Unit::Priest.spawn(placement, *player_num, &mut cmd),
                    Card::Bomber => Unit::Bomber.spawn(placement, *player_num, &mut cmd),
                    Card::Giant => Unit::Giant.spawn(placement, *player_num, &mut cmd),
                },
            }
        }
    }
}

trait DefaultDirection {
    fn default_direction(&self) -> Direction;
}
impl DefaultDirection for PlayerNumber {
    fn default_direction(&self) -> Direction {
        match self {
            PlayerNumber::One => Direction::Up,
            PlayerNumber::Two => Direction::Down,
        }
    }
}

fn calc_direction(direction: &ArenaPos) -> Direction {
    let mut angle = direction.0.acos() * 180. / f32::consts::PI;
    if direction.1 < 0. {
        angle = -angle + 360.;
    }

    match angle {
        0.0..20. | 340.0..360. => Direction::Right,
        20.0..160. => Direction::Up,
        160.0..200. => Direction::Left,
        200.0..340. => Direction::Down,
        _ => Direction::Right,
    }
}

fn sync_entities(
    units: Query<(
        Entity,
        &ArenaPos,
        &UnitState,
        &Attack,
        Option<&Movement>,
        &PlayerNumber,
        &Health,
        Option<&StunnedTimer>,
    )>,
    giants: Query<(
        Entity,
        &ArenaPos,
        &UnitState,
        &Giant,
        &Movement,
        &PlayerNumber,
        &Health,
        Option<&StunnedTimer>,
    )>,
    projectiles: Query<(Entity, &ArenaPos), Without<PlayerNumber>>,
    positions: Query<&ArenaPos>,
    mut server: ResMut<QuinnetServer>,
) {
    let mut u = Vec::new();
    for (entity, pos, state, attack, movement, player_num, health, stun) in &units {
        let direction = match state {
            UnitState::Idle => player_num.default_direction(),
            UnitState::Moving => {
                let movement = movement.unwrap();
                match movement.target {
                    Some(m) => {
                        let Ok(target_pos) = positions.get(m) else {
                            continue;
                        };
                        calc_direction(&pos.direction(target_pos))
                    }
                    None => player_num.default_direction(),
                }
            }
            UnitState::Attacking => match attack.target {
                Some(a) => {
                    let Ok(target_pos) = positions.get(a) else {
                        continue;
                    };
                    calc_direction(&pos.direction(target_pos))
                }
                None => player_num.default_direction(),
            },
        };
        let mut state = *state;
        if let Some(_) = stun {
            state = UnitState::Idle
        }
        u.push((entity, *pos, direction, state, *health));
    }
    for (entity, pos, state, giant, movement, player_num, health, stun) in &giants {
        let direction = match state {
            UnitState::Idle => player_num.default_direction(),
            UnitState::Moving => match movement.target {
                Some(m) => {
                    let Ok(target_pos) = positions.get(m) else {
                        continue;
                    };
                    calc_direction(&pos.direction(target_pos))
                }
                None => player_num.default_direction(),
            },
            UnitState::Attacking => match giant.target {
                Some(a) => {
                    let Ok(target_pos) = positions.get(a) else {
                        continue;
                    };
                    calc_direction(&pos.direction(target_pos))
                }
                None => player_num.default_direction(),
            },
        };
        let mut state = *state;
        if let Some(_) = stun {
            state = UnitState::Idle
        }
        u.push((entity, *pos, direction, state, *health));
    }

    let mut p = Vec::new();
    for (entity, position) in &projectiles {
        p.push((entity, *position));
    }

    server
        .endpoint_mut()
        .broadcast_message_on(
            ServerChannel::Unreliable,
            ServerMessage::SyncEntities {
                units: u,
                projectiles: p,
            },
        )
        .unwrap();
}
