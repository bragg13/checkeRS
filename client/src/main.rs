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
pub enum SceneTransition {
    None,
    ToGame(HashMap<PlayerId, Player>, PlayerId),
    ToMenu,
    ToLobby(String, String),
}
impl Scene {
    pub fn handle_input(&mut self, key_event: KeyEvent) -> SceneTransition {
        match self {
            Scene::Menu(menu) => menu.handle_input(key_event),
            Scene::Game(game_scene) => game_scene.handle_input(key_event),
        }
    }
    pub fn handle_event(&mut self, game_event: GameEvent) -> SceneTransition {
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
}

// events that can trigger re-render in client
pub enum ChannelMessage {
    Input(crossterm::event::KeyEvent),
    ServerMessage(GameEvent),
    ClientIdCommunication(PlayerId),
}

// main thread - TUI rendering on input updates
fn handle_input_events(tx: mpsc::Sender<ChannelMessage>) {
    loop {
        match crossterm::event::read().unwrap() {
            crossterm::event::Event::Key(key_event) => {
                tx.send(ChannelMessage::Input(key_event)).unwrap()
            }
            _ => {}
        }
    }
}
// atm, receives messages from the server and sends them back to the main thread to do the re-rendering
// TODO: client should be able to send messages to the server (communicate own's moves)
fn run_net_thread(tx: mpsc::Sender<ChannelMessage>, username: String, address: String) {
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
    if tx
        .send(ChannelMessage::ClientIdCommunication(client_id))
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
    // let mut connection_timeout = Instant::now();

    loop {
        let now = Instant::now();
        let duration = now - last_updated;
        last_updated = now;

        client.update(duration);
        if let Err(e) = transport.update(duration, &mut client) {
            info!("❌ Transport error: {}", e);
            break;
        };

        // whenever we get a message from server...
        if client.is_connected() {
            // Reset timeout once connected
            // connection_timeout = Instant::now();

            while let Some(message) = client.receive_message(DefaultChannel::ReliableOrdered) {
                // info!("Received message from server {:?}", message);
                // .... if we can deserialize it as a game event....
                match postcard::from_bytes::<GameEvent>(&message) {
                    // .... we send it to the main thread to be handled
                    Ok(game_event) => {
                        if tx.send(ChannelMessage::ServerMessage(game_event)).is_err() {
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
        }
    }

    pub fn run(
        &mut self,
        terminal: &mut DefaultTerminal,
        rx: mpsc::Receiver<ChannelMessage>,
        tx: mpsc::Sender<ChannelMessage>,
    ) -> io::Result<()> {
        while !self.exit {
            // matching events read from a thread channel, ie. coming from input handling, server messages, or internal inter-thread communication
            match rx.recv_timeout(Duration::from_millis(100)) {
                Ok(ChannelMessage::Input(key_event)) => {
                    if key_event.kind == KeyEventKind::Press {
                        if key_event.code == KeyCode::Char('q') {
                            self.exit();
                        } else {
                            match self.current_scene.handle_input(key_event) {
                                // pressing enter in the main menu triggers going to lobby
                                SceneTransition::ToLobby(username, address) => {
                                    let tx_to_net_thread = tx.clone();
                                    thread::spawn(move || {
                                        run_net_thread(tx_to_net_thread, username, address);
                                    });
                                }
                                // not yet implemented, but will be triggered from pressing a button 'start new game'
                                // SceneTransition::ToMenu => {
                                //     self.current_scene = Scene::Menu(MainMenuScene::new()) // after game is finished
                                // }
                                _ => {}
                            }
                        }
                    }
                }
                Ok(ChannelMessage::ClientIdCommunication(client_id)) => self.player_id = client_id,
                Ok(ChannelMessage::ServerMessage(game_event)) => {
                    // this can get triggered when players join (server events)
                    match self.current_scene.handle_event(game_event) {
                        SceneTransition::ToGame(players, starting_player) => {
                            self.current_scene = Scene::Game(GameScene::new(
                                players,
                                self.player_id,
                                starting_player,
                            ));
                        }
                        _ => {}
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

    let (event_tx, event_rx) = mpsc::channel::<ChannelMessage>();
    let tx_to_input_events = event_tx.clone();
    thread::spawn(move || {
        handle_input_events(tx_to_input_events);
    });

    // thread::spawn(move || {
    //     run_net_thread(tx_to_net_thread);
    // });

    // UI takes receiver channel from which other threads communicate
    ratatui::run(|terminal| App::new().run(terminal, event_rx, event_tx)) // TODO: change the player id
}
