use std::{
    io,
    net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket},
    sync::mpsc,
    thread,
    time::{Duration, Instant, SystemTime},
};

use cli_log::{LevelFilter, info, trace};
use crossterm::event::{self, KeyCode, KeyEventKind};
use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    layout::{
        Constraint::{self, Length},
        Layout, Margin, Rect,
    },
    style::{Color, Stylize},
    symbols::Marker,
    text::Line,
    widgets::{
        Block, Paragraph, Widget,
        canvas::{Canvas, Circle},
    },
};
use renet::{Bytes, ClientId, ConnectionConfig, DefaultChannel, RenetClient};
use renet_netcode::{
    ClientAuthentication, NETCODE_USER_DATA_BYTES, NetcodeClientTransport, NetcodeServerTransport,
    ServerAuthentication, ServerConfig,
};
use store::{
    CELL_N, PROTOCOL_ID,
    coords::Coords,
    game_state::{GameEvent, GameState},
    game_utils::{Move, coords_to_index, get_possible_moves, is_white},
    player::PlayerId,
};

#[derive(Debug)]
pub struct App {
    possible_moves: Vec<Move>,
    cursor_cell: Coords,
    game_state: GameState,
    selected_cell: Option<Coords>,
    player_id: PlayerId,
    exit: bool,
}

// events that can trigger re-render in client
// TODO: change name
enum ClientEvent {
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
    pub fn new(player_id: PlayerId) -> Self {
        Self {
            game_state: GameState::new(),
            cursor_cell: Coords { x: 0, y: 0 },
            selected_cell: None,
            exit: false,
            player_id,
            possible_moves: vec![],
        }
    }

    pub fn run(
        &mut self,
        terminal: &mut DefaultTerminal,
        rx: mpsc::Receiver<ClientEvent>,
    ) -> io::Result<()> {
        while !self.exit {
            match rx.recv().unwrap() {
                ClientEvent::Input(key_event) => {
                    if key_event.kind == KeyEventKind::Press {
                        match key_event.code {
                            KeyCode::Char('q') => self.exit(),
                            KeyCode::Char('h') => self.left(),
                            KeyCode::Char('j') => self.down(),
                            KeyCode::Char('k') => self.up(),
                            KeyCode::Char('l') => self.right(),
                            KeyCode::Char(' ') => self.select(),
                            _ => {}
                        }
                    }
                }
                ClientEvent::ServerMessage(bytes) => {
                    info!("{:?}", bytes);
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
    fn left(&mut self) {
        if self.cursor_cell.x != 0 {
            self.cursor_cell.x -= 1;
        }
    }
    fn down(&mut self) {
        if self.cursor_cell.y < CELL_N - 1 {
            self.cursor_cell.y += 1;
        }
    }
    fn up(&mut self) {
        if self.cursor_cell.y > 0 {
            self.cursor_cell.y -= 1;
        }
    }
    fn right(&mut self) {
        if self.cursor_cell.x != CELL_N - 1 {
            self.cursor_cell.x += 1;
        }
    }
    fn select(&mut self) {
        if !is_white(self.cursor_cell) {
            return;
        }

        // selecting empty cell
        if self.game_state.grid[self.cursor_cell].is_none() {
            let selected_move = self
                .possible_moves
                .iter()
                .find(|possible_move| possible_move.to() == self.cursor_cell);
            match selected_move {
                Some(mv) => {
                    // move selected pawn to selected cell
                    let event = GameEvent::Move {
                        mv: selected_move.unwrap().clone(),
                        player_id: self.player_id,
                    };
                    self.game_state.dispatch(&event);
                    self.possible_moves.clear();
                    self.selected_cell = None;
                }
                None => {}
            }
        }

        // selecting our own pawn
        if self.game_state.grid[self.cursor_cell].is_some_and(|x| x.player_id == self.player_id) {
            self.selected_cell = Some(self.cursor_cell);
            let moves = get_possible_moves(
                &self.game_state.grid,
                self.selected_cell.unwrap(),
                self.game_state.players.get(&self.player_id).unwrap(), // TODO - BREAKS
            );
            self.possible_moves = moves;
        }
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

        let vertical_layout =
            Layout::vertical([Constraint::Percentage(8), Constraint::Percentage(92)]).spacing(1);
        let [info_area, board_area] = vertical_layout.areas(area.inner(Margin::new(1, 1)));

        // info area // TODO
        if let Some(player1) = self.game_state.players.get(&(1 as PlayerId))
            && let Some(player2) = self.game_state.players.get(&(2 as PlayerId))
        {
            Paragraph::new(vec![
                player1.pretty_print_scoreboard().left_aligned(),
                player2.pretty_print_scoreboard().left_aligned(),
            ])
            .render(info_area, buf);
        };

        // board
        let cell_size = board_area.height / 8;
        let rows = Layout::vertical([Length(cell_size); 8])
            .flex(ratatui::layout::Flex::Start)
            .split(board_area);

        let cells = rows
            .iter()
            .flat_map(|row| {
                Layout::horizontal([Length(cell_size * 2); 8])
                    .flex(ratatui::layout::Flex::Center)
                    .split(*row)
                    .iter()
                    .copied()
                    .take(8)
                    .collect::<Vec<Rect>>()
            })
            .collect::<Vec<_>>();

        for i in 0..CELL_N {
            for j in 0..CELL_N {
                let coords = Coords { x: i, y: j };
                let c = &Circle {
                    x: 5.0,
                    y: 5.0,
                    color: if self.game_state.grid[coords]
                        .is_some_and(|x| x.player_id == self.player_id)
                    // TODO?
                    {
                        Color::Green // player
                    } else {
                        Color::Red // opponent
                    },
                    radius: 5.0,
                };

                let cell_color = if coords == self.cursor_cell {
                    Color::LightGreen
                } else if let Some(selected_cell) = self.selected_cell
                    && coords == selected_cell
                {
                    Color::Yellow
                } else if self
                    .possible_moves
                    .iter()
                    .map(|cell| cell.to())
                    .collect::<Vec<_>>()
                    .contains(&coords)
                {
                    Color::LightYellow
                } else {
                    if is_white(coords) {
                        Color::White
                    } else {
                        Color::Black
                    }
                };

                Canvas::default()
                    .block(
                        Block::bordered().bg(cell_color).fg(cell_color), //.title(format!("{:?}-{:?}-{:?}", i, j, coords_to_index(coords))),
                    )
                    .marker(Marker::Braille)
                    .background_color(cell_color)
                    .x_bounds([0.0, 10.0])
                    .y_bounds([0.0, 10.0])
                    .paint(|ctx| {
                        if self.game_state.grid[coords].is_some_and(|x| x.player_id > 0) {
                            ctx.draw(c);
                        }
                    })
                    .render(cells[coords_to_index(coords)], buf);
            }
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
    ratatui::run(|terminal| App::new(1).run(terminal, event_rx)) // TODO: change the player id
}
