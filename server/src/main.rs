use cli_log::{info, trace};
use renet::{ConnectionConfig, DefaultChannel, RenetServer, ServerEvent};
use renet_netcode::{NetcodeServerTransport, ServerAuthentication, ServerConfig};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket};
use std::time::{Duration, Instant, SystemTime};
use store::{CHANNEL_ID, PROTOCOL_ID};

fn main() {
    env_logger::Builder::from_default_env()
        .filter_level(cli_log::LevelFilter::Trace) // Show all logs
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
    trace!("🕹 server listening on {}", SERVER_ADDR);

    loop {
        let now = Instant::now();
        let duration = now - last_updated;
        last_updated = now;
        server.update(duration);
        transport.update(duration, &mut server).unwrap();
        trace!("🕹 server looping...");

        // handles events
        while let Some(event) = server.get_event() {
            match event {
                ServerEvent::ClientConnected { client_id } => {
                    info!("Client connected! {}", client_id);
                }
                ServerEvent::ClientDisconnected { client_id, .. } => {
                    info!(":( Client disconnected! {}", client_id);
                }
            }
        }

        // receive messges
        for client_id in server.clients_id() {
            while let Some(msg) = server.receive_message(client_id, DefaultChannel::ReliableOrdered)
            {
                info!("Received message from {} saying {:?}.", client_id, msg);

                server.send_message(client_id, CHANNEL_ID, msg);
                // server.send_message or handling...
            }
        }

        transport.send_packets(&mut server);
        std::thread::sleep(Duration::from_millis(1000));
    }
}
