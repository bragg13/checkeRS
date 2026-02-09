mod game;
mod main_menu;
use std::{
    io,
    net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket},
    sync::mpsc,
    thread,
    time::{Duration, Instant, SystemTime},
};

use cli_log::{LevelFilter, info, trace};
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    layout::Rect,
    style::Stylize,
    text::Line,
    widgets::{Block, Widget},
};
use renet::{Bytes, ClientId, ConnectionConfig, DefaultChannel, RenetClient};
use renet_netcode::{ClientAuthentication, NetcodeClientTransport};
use store::PROTOCOL_ID;

use crate::{game::GameScene, main_menu::MainMenuScene};

#[derive(Debug)]
enum Scene {
    Menu(MainMenuScene),
    Game(GameScene),
}
impl Scene {
    pub fn handle_input(&mut self, key_event: KeyEvent) {
        match self {
            Scene::Menu(menu) => menu.handle_input(key_event),
            Scene::Game(game) => game.handle_input(key_event),
        }
    }
}

#[derive(Debug)]
pub struct App {
    exit: bool,
    scenes: Vec<Scene>,
    current_scene: usize,
}

// events that can trigger re-render in client
// TODO: change name
pub enum ClientEvent {
    Input(crossterm::event::KeyEvent),
    ServerMessage(Bytes), // TODO: will be GameEvent
}

// main thread - TUI rendering on input updates
fn handle_input_events(tx: mpsc::Sender<ClientEvent>) {
    loop {
        match crossterm::event::read().unwrap() {
            crossterm::event::Event::Key(key_event) => {
                tx.send(ClientEvent::Input(key_event)).unwrap()
            }
            _ => {}
        }
    }
}
// atm, receives messages from the server and sends them back to the main thread to do the re-rendering
// TODO: client should be able to send messages to the server (communicate own's moves)
fn run_net_thread(tx: mpsc::Sender<ClientEvent>) {
    const SERVER_ADDR: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 5000);
    const CLIENT_ADDR: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 0);

    let socket = UdpSocket::bind(CLIENT_ADDR).unwrap();
    let current_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();

    let client_id = ClientId::from(current_time.as_millis() as u64);
    let mut client = RenetClient::new(ConnectionConfig::default());

    let auth = ClientAuthentication::Unsecure {
        client_id,
        protocol_id: PROTOCOL_ID,
        server_addr: SERVER_ADDR,
        user_data: None,
    };
    let mut transport = NetcodeClientTransport::new(current_time, auth, socket).unwrap();
    info!("Client connecting to {}", SERVER_ADDR);
    let mut last_updated = Instant::now();

    loop {
        let now = Instant::now();
        let duration = now - last_updated;
        last_updated = now;

        client.update(duration);
        transport.update(duration, &mut client).unwrap();

        if client.is_connected() {
            while let Some(message) = client.receive_message(DefaultChannel::ReliableOrdered) {
                info!("Received message from server {:?}", message);
                tx.send(ClientEvent::ServerMessage(message)).unwrap();
            }
        }
        transport.send_packets(&mut client).unwrap();
    }
}

/// ====== MAIN APP =======
impl App {
    pub fn new() -> Self {
        Self {
            exit: false,
            // there has to be a better way
            scenes: vec![
                Scene::Menu(MainMenuScene::new()),
                Scene::Game(GameScene::new()),
            ],
            current_scene: 0,
        }
    }

    pub fn run(
        &mut self,
        terminal: &mut DefaultTerminal,
        rx: mpsc::Receiver<ClientEvent>,
    ) -> io::Result<()> {
        while !self.exit {
            match rx.recv_timeout(Duration::from_millis(100)) {
                Ok(ClientEvent::Input(key_event)) => {
                    if key_event.kind == KeyEventKind::Press {
                        if key_event.code == KeyCode::Char('q') {
                            self.exit();
                        } else if let Some(scene) = self.scenes.get_mut(self.current_scene) {
                            scene.handle_input(key_event);
                        }
                    }
                }
                Ok(ClientEvent::ServerMessage(bytes)) => {
                    info!("{:?}", bytes);
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
            "<HJKL>".blue().bold(),
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

        // TODO: can maybe move to Scene impl?
        match &self.scenes[self.current_scene] {
            Scene::Menu(main_menu_scene) => main_menu_scene.render(area, buf),
            Scene::Game(game_scene) => game_scene.render(area, buf),
        }
    }
}

fn main() -> io::Result<()> {
    env_logger::Builder::from_default_env()
        .filter_level(LevelFilter::Info) // Show all logs
        .init();

    let (event_tx, event_rx) = mpsc::channel::<ClientEvent>();
    let tx_to_input_events = event_tx.clone();
    thread::spawn(move || {
        handle_input_events(tx_to_input_events);
    });

    let tx_to_net_thread = event_tx.clone();
    thread::spawn(move || {
        run_net_thread(tx_to_net_thread);
    });

    // UI takes receiver channel from which other threads communicate
    ratatui::run(|terminal| App::new().run(terminal, event_rx)) // TODO: change the player id
}
