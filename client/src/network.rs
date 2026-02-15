use std::{
    sync::mpsc,
    time::{Duration, Instant, SystemTime},
};

use cli_log::info;
use renet::{ClientId, ConnectionConfig, DefaultChannel, RenetClient};
use renet_netcode::{ClientAuthentication, NetcodeClientTransport};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket};
use store::{PROTOCOL_ID, game_state::GameEvent, utils::to_netcode_user_data};

use crate::{ClientToServerMessage, IncomingEvent};

pub fn run_net_thread(
    network_to_main_tx: mpsc::Sender<IncomingEvent>,
    main_to_network_rx: mpsc::Receiver<ClientToServerMessage>,
    username: String,
    address: String,
) {
    let server_addr: SocketAddr = match address.parse() {
        Ok(addr) => addr,
        Err(e) => {
            info!("❌ Invalid address '{}': {}", address, e);
            return;
        }
    };
    let client_addr: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 0);

    let socket = match UdpSocket::bind(client_addr) {
        Ok(s) => s,
        Err(e) => {
            info!("❌ Failed to bind socket: {}", e);
            return;
        }
    };
    let current_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();

    let client_id = ClientId::from(current_time.as_millis() as u64);
    if network_to_main_tx
        .send(IncomingEvent::ClientIdCommunication(client_id))
        .is_err()
    {
        info!("❌ Failed to communicate client id to main thread");
        return;
    }
    let mut client = RenetClient::new(ConnectionConfig::default());

    let auth = ClientAuthentication::Unsecure {
        client_id,
        protocol_id: PROTOCOL_ID,
        server_addr: server_addr,
        user_data: Some(to_netcode_user_data(username)),
    };

    let mut transport = match NetcodeClientTransport::new(current_time, auth, socket) {
        Ok(t) => t,
        Err(e) => {
            info!("❌ Failed to create transport: {}", e);
            return;
        }
    };

    let mut last_updated = Instant::now();

    loop {
        let now = Instant::now();
        let duration = now - last_updated;
        last_updated = now;

        client.update(duration);
        if let Err(e) = transport.update(duration, &mut client) {
            info!("❌ Transport error: {}", e);
            break;
        };

        // get Move instruction from input thread and send to server
        while let Ok(command) = main_to_network_rx.try_recv() {
            match command {
                ClientToServerMessage::SendEvent(game_event) => match game_event {
                    GameEvent::EndGame { .. } => {
                        info!("Game ended, exiting network thread...");
                        break;
                    }
                    GameEvent::Move { .. } => match postcard::to_allocvec(&game_event) {
                        Ok(bytes) => client.send_message(DefaultChannel::ReliableOrdered, bytes),
                        Err(_) => {
                            info!("Error while serializing game event")
                        }
                    },
                    _ => {}
                },
            }
        }

        // event from server
        if client.is_connected() {
            while let Some(message) = client.receive_message(DefaultChannel::ReliableOrdered) {
                match postcard::from_bytes::<GameEvent>(&message) {
                    // .... we send it to the main thread to be handled
                    Ok(game_event) => {
                        if network_to_main_tx
                            .send(IncomingEvent::ServerMessage(game_event))
                            .is_err()
                        {
                            info!("❌ Main thread closed, exiting network thread");
                            break;
                        }
                    }
                    Err(e) => {
                        info!("⚠️ Failed to deserialize message: {}", e);
                    }
                }
            }
        }
        if let Err(e) = transport.send_packets(&mut client) {
            info!("❌ Failed to send packets: {}", e);
            break;
        }
        std::thread::sleep(Duration::from_millis(16));
    }
    info!("🔌 Network thread exiting");
}
