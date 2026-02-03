use std::{collections::HashMap, io};

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
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

use crate::{
    coords::Coords,
    game_state::{GameEvent, GameState},
    game_utils::{Move, coords_to_index, get_possible_moves, is_white},
    piece::{Piece, PieceType},
    player::PlayerId,
};
mod board;
mod coords;
mod game_state;
mod game_utils;
mod piece;
mod player;

static CELL_N: usize = 8;

#[derive(Debug)]
pub struct App {
    possible_moves: Vec<Move>,
    cursor_cell: Coords,
    game_state: GameState,
    selected_cell: Option<Coords>,
    player_id: PlayerId,
    exit: bool,
}

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

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                // if self.is_turn == self.player_id {
                match key_event.code {
                    KeyCode::Char('q') => self.exit(),
                    KeyCode::Char('h') => self.left(),
                    KeyCode::Char('j') => self.down(),
                    KeyCode::Char('k') => self.up(),
                    KeyCode::Char('l') => self.right(),
                    KeyCode::Char(' ') => self.select(),
                    _ => {}
                }
                // }
            }
            _ => {}
        };
        Ok(())
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
        if self.grid[self.cursor_cell].is_none() {
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
        if self.grid[self.cursor_cell].is_some_and(|x| x.player_id == self.is_turn) {
            self.selected_cell = Some(self.cursor_cell);
            let moves = get_possible_moves(
                &self.grid,
                self.selected_cell.unwrap(),
                self.players.get(&self.is_turn).unwrap(),
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

        // info area
        if let Some(player1) = self.players.get(&(1 as PlayerId))
            && let Some(player2) = self.players.get(&(2 as PlayerId))
        {
            Paragraph::new(vec![
                player1.pretty_print_scoreboard().left_aligned(),
                player2.pretty_print_scoreboard().left_aligned(),
                Line::from(format!("player {:?} is playing", self.is_turn)).right_aligned(),
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
                    color: if self.grid[coords].is_some_and(|x| x.player_id == self.is_turn) {
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
                        if self.grid[coords].is_some_and(|x| x.player_id > 0) {
                            ctx.draw(c);
                        }
                    })
                    .render(cells[coords_to_index(coords)], buf);
            }
        }
    }
}

fn main() -> io::Result<()> {
    cli_log::init_cli_log!();
    ratatui::run(|terminal| App::new(1).run(terminal)) // TODO: change the player id
}
