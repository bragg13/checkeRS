mod game;
mod main_menu;
use std::{
    collections::HashMap,
    io,
    net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket},
    sync::mpsc,
    thread,
    time::{Duration, Instant, SystemTime},
};

use cli_log::{LevelFilter, info};
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    layout::Rect,
    style::Stylize,
    text::Line,
    widgets::{Block, Widget},
};
use renet::{ClientId, ConnectionConfig, DefaultChannel, RenetClient};
use renet_netcode::{ClientAuthentication, NetcodeClientTransport};
use store::{
    PROTOCOL_ID,
    game_state::GameEvent,
    player::{Player, PlayerId},
    utils::to_netcode_user_data,
};

use crate::{game::GameScene, main_menu::MainMenuScene};

#[derive(Debug)]
pub enum Scene {
    Menu(MainMenuScene),
    Game(GameScene),
}
#[derive(Debug)]
pub enum ClientEvent {
    GoToGame(HashMap<PlayerId, Player>, PlayerId),
    GoToMenu,
    GoToLobby(String, String),
    SendToServer(GameEvent),
}
impl Scene {
    pub fn handle_input(&mut self, key_event: KeyEvent) -> Option<ClientEvent> {
        match self {
            Scene::Menu(menu) => menu.handle_input(key_event),
            Scene::Game(game_scene) => game_scene.handle_input(key_event),
        }
    }
    pub fn handle_event(&mut self, game_event: GameEvent) -> Option<ClientEvent> {
        match self {
            Scene::Menu(menu) => menu.handle_server_events(game_event),
            Scene::Game(game_scene) => game_scene.handle_server_events(game_event),
        }
    }
    pub fn handle_render(&self, area: Rect, buf: &mut Buffer) {
        match self {
            Scene::Menu(main_menu_scene) => main_menu_scene.render(area, buf),
            Scene::Game(game_scene) => game_scene.render(area, buf),
        }
    }
}

#[derive(Debug)]
pub struct App {
    exit: bool,
    current_scene: Scene,
    player_id: PlayerId,
    main_to_network_tx: Option<mpsc::Sender<ClientToServerMessage>>,
}

// cliewnt has to handle this; can come from server or input; is sent via thread channel
pub enum IncomingEvent {
    Input(crossterm::event::KeyEvent),
    ServerMessage(GameEvent),
    ClientIdCommunication(PlayerId),
}

// events that the client can send to the server
pub enum ClientToServerMessage {
    SendEvent(GameEvent),
    Disconnect, // this could be a game event maybe
}

fn handle_input_events(tx: mpsc::Sender<IncomingEvent>) {
    loop {
        match crossterm::event::read().unwrap() {
            crossterm::event::Event::Key(key_event) => {
                tx.send(IncomingEvent::Input(key_event)).unwrap()
            }
            _ => {}
        }
    }
}

fn run_net_thread(
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
                ClientToServerMessage::SendEvent(game_event) => {
                    match postcard::to_allocvec(&game_event) {
                        Ok(bytes) => client.send_message(DefaultChannel::ReliableOrdered, bytes),
                        Err(_) => {
                            info!("Error while serializing game event")
                        }
                    }
                }
                ClientToServerMessage::Disconnect => {
                    return;
                }
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

/// ====== MAIN APP =======
impl App {
    pub fn new() -> Self {
        Self {
            exit: false,
            current_scene: Scene::Menu(MainMenuScene::new()),
            player_id: 0,
            main_to_network_tx: None,
        }
    }

    pub fn run(
        &mut self,
        terminal: &mut DefaultTerminal,
        rx: mpsc::Receiver<IncomingEvent>,
        tx: mpsc::Sender<IncomingEvent>,
    ) -> io::Result<()> {
        while !self.exit {
            // matching events read from a thread channel, ie. coming from input handling, server messages, or internal inter-thread communication
            match rx.recv_timeout(Duration::from_millis(100)) {
                Ok(IncomingEvent::ClientIdCommunication(client_id)) => self.player_id = client_id, // TODO lets find a better way
                // stuff triggered by input
                Ok(IncomingEvent::Input(key_event)) => {
                    if key_event.kind == KeyEventKind::Press {
                        if key_event.code == KeyCode::Char('q') {
                            if let Some(tx) = &self.main_to_network_tx {
                                // send to network thread via channel
                                let _ = tx.send(ClientToServerMessage::Disconnect);
                            }
                            self.exit();
                        } else {
                            if let Some(key_event) = self.current_scene.handle_input(key_event) {
                                match key_event {
                                    ClientEvent::GoToLobby(username, address) => {
                                        // will go to network thread to communicate back server events
                                        let network_to_main_tx = tx.clone();

                                        // rx will go to network thread to receive client actions to be sent to server
                                        // tx will go to App, so we can send client actions from later on in the loop
                                        let (main_to_network_tx, main_to_network_rx) =
                                            mpsc::channel();
                                        self.main_to_network_tx = Some(main_to_network_tx);

                                        thread::spawn(move || {
                                            run_net_thread(
                                                network_to_main_tx,
                                                main_to_network_rx,
                                                username,
                                                address,
                                            );
                                        });
                                    }
                                    // when selecting a cell to move to as a client
                                    ClientEvent::SendToServer(game_event) => {
                                        if let Some(tx) = &self.main_to_network_tx {
                                            match tx
                                                .send(ClientToServerMessage::SendEvent(game_event))
                                            {
                                                Ok(_) => {
                                                    // actuate move
                                                }
                                                Err(_) => {}
                                            }
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                }
                // stuff triggered by server payload
                Ok(IncomingEvent::ServerMessage(msg)) => {
                    if let Some(game_event) = self.current_scene.handle_event(msg) {
                        match game_event {
                            ClientEvent::GoToGame(players, starting_player) => {
                                self.current_scene = Scene::Game(GameScene::new(
                                    players,
                                    self.player_id,
                                    starting_player,
                                ))
                            }
                            _ => {}
                        }
                    }
                }
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    info!("channel closed, exiting...");
                    break;
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    // IMPORTANT: just continue in case of timeout.
                    // `rx.recv()` would be a blocking instruction
                    // `rx.recv_timeout()` allows me to check the loop condition for self.exit
                }
            };
            terminal.draw(|frame| self.draw(frame))?;
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn exit(&mut self) {
        self.exit = true;
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = Line::from("Checkers game").centered();
        let instructions = Line::from(vec![
            "Move ".into(),
            "<Arrows>".blue().bold(),
            " Select ".into(),
            "<space>".blue().bold(),
            " Quit ".into(),
            "<Q>".red().bold(),
        ])
        .centered();

        Block::bordered()
            .title(title)
            .title_bottom(instructions)
            .render(area, buf);

        self.current_scene.handle_render(area, buf);
    }
}

fn main() -> io::Result<()> {
    env_logger::Builder::from_default_env()
        .filter_level(LevelFilter::Info) // Show all logs
        .init();

    let (event_tx, event_rx) = mpsc::channel::<IncomingEvent>();
    let tx_to_input_events = event_tx.clone();
    thread::spawn(move || {
        handle_input_events(tx_to_input_events);
    });

    // UI takes receiver channel from which other threads communicate
    ratatui::run(|terminal| App::new().run(terminal, event_rx, event_tx))
}
