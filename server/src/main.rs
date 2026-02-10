use cli_log::info;
use renet::{ConnectionConfig, DefaultChannel, RenetServer, ServerEvent};
use renet_netcode::{NetcodeServerTransport, ServerAuthentication, ServerConfig};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket};
use std::time::{Duration, Instant, SystemTime};
use store::game_state::GameEvent;
use store::player::{Player, PlayerId};
use store::utils::from_user_data;
use store::{CHANNEL_ID, PROTOCOL_ID};

fn main() {
    env_logger::Builder::from_default_env()
        .filter_level(cli_log::LevelFilter::Info) // Show all logs
        .init();
    let mut server = RenetServer::new(ConnectionConfig::default());

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
                    let joined_event = GameEvent::PlayerJoined {
                        player: Player {
                            id: username.len() as PlayerId,
                            name: username,
                            direction: 1,
                            score: 0,
                        },
                    };
                    let bytes = postcard::to_allocvec(&joined_event).unwrap();
                    server.broadcast_message(DefaultChannel::ReliableOrdered, bytes);
                }
                ServerEvent::ClientDisconnected { client_id, reason } => {
                    info!(":( Client disconnected! {client_id}, reason: {reason}");
                    // server.broadcast_message(
                    //     DefaultChannel::ReliableOrdered,
                    //     b"a player disconnected", // TODO: make this an event
                    // );
                }
            }
        }

        // receive messges from channel
        // for client_id in server.clients_id() {
        //     // default channel is the one used in the configuration
        // }

        transport.send_packets(&mut server);
        std::thread::sleep(Duration::from_millis(16));
    }
}
