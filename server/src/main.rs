use cli_log::info;
use renet::{ConnectionConfig, DefaultChannel, RenetServer, ServerEvent};
use renet_netcode::{
    ClientAuthentication, NetcodeServerTransport, ServerAuthentication, ServerConfig,
};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket};
use std::time::{Duration, Instant, SystemTime};
use store::game_state::{self, GameEvent, GameState};
use store::player::{Player, PlayerId};
use store::utils::from_user_data;
use store::{CHANNEL_ID, PROTOCOL_ID};

fn main() {
    env_logger::Builder::from_default_env()
        .filter_level(cli_log::LevelFilter::Info) // Show all logs
        .init();
    let mut server = RenetServer::new(ConnectionConfig::default());
    let mut game_state = GameState::new(None);

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

                    // notify all players that a new player joined
                    let player = Player {
                        id: client_id as PlayerId,
                        name: username,
                        direction: if server.connected_clients() == 2 {
                            -1
                        } else {
                            1
                        },
                        score: 0,
                    };
                    let joined_event = GameEvent::PlayerJoined {
                        player: player.clone(),
                    };
                    let bytes = postcard::to_allocvec(&joined_event).unwrap();
                    info!("broadcasting that a new player {} joined...", player.name);
                    server.broadcast_message(DefaultChannel::ReliableOrdered, bytes);

                    // if this is the second player to connect, also notiufy him about the first one
                    if server.connected_clients() == 2 {
                        for prev_client in game_state.players.iter() {
                            let joined_event = GameEvent::PlayerJoined {
                                player: prev_client.1.clone(), // TODO: probably not the best
                            };
                            let bytes = postcard::to_allocvec(&joined_event).unwrap();
                            server.send_message(client_id, DefaultChannel::ReliableOrdered, bytes);
                            info!(
                                "telling {} that about player {} ...",
                                prev_client.0, prev_client.1.name
                            );
                        }
                    }

                    // add player to game state
                    game_state.players.insert(client_id, player);
                }
                ServerEvent::ClientDisconnected { client_id, reason } => {
                    info!(":( Client disconnected! {client_id}, reason: {reason}");
                }
            }
        }

        // receive messges from channel
        for client_id in server.clients_id() {
            // default channel is the one used in the configuration
        }

        transport.send_packets(&mut server);
        std::thread::sleep(Duration::from_millis(16));
    }
}
