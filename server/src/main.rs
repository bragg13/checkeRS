use renet::{
    ConnectionConfig, NETCODE_USER_DATA_BYTES, RenetConnectionConfig, RenetServer,
    ServerAuthentication, ServerConfig, ServerEvent,
};
use renet_netcode::{
    ClientAuthentication, NETCODE_USER_DATA_BYTES, NetcodeClientTransport, NetcodeServerTransport,
    ServerAuthentication, ServerConfig,
};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket};
use std::time::{Duration, Instant, SystemTime};

fn main() {
    let mut server = RenetServer::new(ConnectionConfig::default());

    // Setup transport layer using renet_netcode
    const SERVER_ADDR: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 5000);
    let socket: UdpSocket = UdpSocket::bind(SERVER_ADDR).unwrap();
    let server_config = ServerConfig {
        current_time: SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap(),
        max_clients: 2,
        protocol_id: 0,
        public_addresses: vec![SERVER_ADDR],
        authentication: ServerAuthentication::Unsecure,
    };
    let mut transport = NetcodeServerTransport::new(server_config, socket).unwrap();
    let mut last_updated = Instant::now();

    loop {
        let now = Instant::now();
        let duration = now - last_updated;
        last_updated = now;
        server.update(duration);
        transport.update(duration, &mut server).unwrap();

        while let Some(event) = server.get_event() {
            match event {
                ServerEvent::ClientConnected { client_id } => todo!(),
                ServerEvent::ClientDisconnected { client_id, reason } => todo!(),
            }
        }
    }
}
