use cli_log::info;
use renet::{ConnectionConfig, DefaultChannel, RenetServer, ServerEvent};
use renet_netcode::{NetcodeServerTransport, ServerAuthentication, ServerConfig};
use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket};
use std::time::{Duration, Instant, SystemTime};
use store::PROTOCOL_ID;
use store::game_state::{GameEvent, GameState};
use store::player::{Player, PlayerId};
use store::utils::from_user_data;

fn main() {
    env_logger::Builder::from_default_env()
        .filter_level(cli_log::LevelFilter::Info) // Show all logs
        .init();
    let mut server = RenetServer::new(ConnectionConfig::default());
    let mut game_state: Option<GameState> = None;

    // Setup transport layer using renet_netcode
    const SERVER_ADDR: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 5000);
    let socket: UdpSocket = UdpSocket::bind(SERVER_ADDR).unwrap();
    let server_config = ServerConfig {
        current_time: SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap(),
        max_clients: 2,
        protocol_id: PROTOCOL_ID,
        public_addresses: vec![SERVER_ADDR],
        authentication: ServerAuthentication::Unsecure,
    };
    let mut transport = NetcodeServerTransport::new(server_config, socket).unwrap();
    let mut last_updated = Instant::now();
    let mut starting_player_id: Option<PlayerId> = None;
    let mut players = HashMap::new();
    info!("🕹 server listening on {}", SERVER_ADDR);

    loop {
        let now = Instant::now();
        let duration = now - last_updated;
        last_updated = now;
        server.update(duration);
        transport.update(duration, &mut server).unwrap();
        // info!("🕹 server looping...");

        // handles events
        while let Some(event) = server.get_event() {
            match event {
                ServerEvent::ClientConnected { client_id } => {
                    let user_data = transport.user_data(client_id).unwrap();
                    let username = from_user_data(&user_data);
                    info!("Client connected! {} with username {}", client_id, username);
                    if server.connected_clients() < 2 {
                        starting_player_id = Some(client_id);
                    }

                    // notify all players that a new player joined
                    let player = Player {
                        id: client_id as PlayerId,
                        name: username,
                        direction: if starting_player_id.is_some_and(|id| id == client_id) {
                            1
                        } else {
                            -1
                        },
                        score: 0,
                    };
                    let joined_event = GameEvent::PlayerJoined {
                        player: player.clone(),
                    };
                    let bytes = postcard::to_allocvec(&joined_event).unwrap();
                    info!("broadcasting that a new player {} joined...", player.name);
                    server.broadcast_message(DefaultChannel::ReliableOrdered, bytes);

                    // add player to game state
                    players.insert(client_id, player);

                    // if this is the second player to connect, also notify him about the first one
                    // TODO: i really doint like this. maybe its better to send the whole players list before game starts?
                    if server.connected_clients() == 2 {
                        let prev_player_joined_event = GameEvent::PlayerJoined {
                            player: players.get(&starting_player_id.unwrap()).unwrap().clone(),
                        };
                        let bytes = postcard::to_allocvec(&prev_player_joined_event).unwrap();
                        server.send_message(client_id, DefaultChannel::ReliableOrdered, bytes); // sending to the newly connected client

                        info!("starting the game...");
                        let start_game = GameEvent::TurnChanged {
                            player_id: starting_player_id.unwrap(),
                        };
                        let bytes = postcard::to_allocvec(&start_game).unwrap();
                        server.broadcast_message(DefaultChannel::ReliableOrdered, bytes);

                        // init server state
                        game_state =
                            Some(GameState::new(players.clone(), starting_player_id.unwrap()));
                    }
                }
                ServerEvent::ClientDisconnected { client_id, reason } => {
                    info!(":( Client disconnected! {client_id}, reason: {reason}");
                }
            }
        }

        // receive messges from channel
        for client_id in server.clients_id() {
            info!("received a move from client {:?}", client_id);
        }

        transport.send_packets(&mut server);
        std::thread::sleep(Duration::from_millis(16));
    }
}
