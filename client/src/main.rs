mod game;
mod main_menu;
mod network;
mod scene;
use std::{
    io,
    sync::mpsc::{self, RecvTimeoutError},
    thread,
    time::Duration,
};

use cli_log::{LevelFilter, info};
use crossterm::event::{KeyCode, KeyEventKind};
use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    layout::Rect,
    style::Stylize,
    text::Line,
    widgets::{Block, Widget},
};
use store::{
    game_state::{ClientEvent, GameEvent},
    player::PlayerId,
};

use crate::{game::GameScene, main_menu::MainMenuScene, network::run_net_thread, scene::Scene};

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

impl App {
    pub fn new() -> Self {
        Self {
            exit: false,
            current_scene: Scene::Menu(MainMenuScene::new(None)),
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
                                let _ = tx.send(ClientToServerMessage::SendEvent(
                                    GameEvent::PlayerLeft {
                                        player_id: self.player_id,
                                    },
                                ));
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
                                    ClientEvent::SendToServer(game_event) => {
                                        if let Some(tx) = &self.main_to_network_tx {
                                            if tx
                                                .send(ClientToServerMessage::SendEvent(game_event))
                                                .is_err()
                                            {
                                                info!(
                                                    "❌ Something happened while sending event to server..."
                                                )
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
                            ClientEvent::GoToMenu(end_game_reason) => {
                                // disconnect the net thread, delete channel, and go to menu
                                if let Some(tx) = &self.main_to_network_tx {
                                    if tx
                                        .send(ClientToServerMessage::SendEvent(
                                            GameEvent::EndGame {
                                                reason: end_game_reason.clone(),
                                            },
                                        ))
                                        .is_err()
                                    {
                                        info!("❌ Something happened while going to menu...")
                                    }
                                }
                                self.main_to_network_tx = None;

                                self.current_scene =
                                    Scene::Menu(MainMenuScene::new(Some(end_game_reason.clone())));
                            }
                            ClientEvent::GoToLobby(_, _) => todo!(),
                            ClientEvent::SendToServer(_game_event) => todo!(),
                        }
                    }
                }
                Err(RecvTimeoutError::Disconnected) => {
                    info!("🔌 channel closed, exiting...");
                    self.exit();
                    break;
                }
                Err(RecvTimeoutError::Timeout) => {
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
        .filter_level(LevelFilter::Info)
        .init();

    let (event_tx, event_rx) = mpsc::channel::<IncomingEvent>();
    let tx_to_input_events = event_tx.clone();
    thread::spawn(move || {
        handle_input_events(tx_to_input_events);
    });

    ratatui::run(|terminal| App::new().run(terminal, event_rx, event_tx))
}
