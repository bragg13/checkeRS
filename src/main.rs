use std::io;

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
    board::Board,
    coords::Coords,
    game_utils::{coords_to_index, get_possible_moves, is_white},
    piece::{Piece, PieceType},
    player::Player,
};
mod board;
mod coords;
mod game_utils;
mod piece;
mod player;

static CELL_N: usize = 8;

#[derive(Debug)]
pub struct App {
    grid: Board,
    is_turn: usize,
    cursor_cell: Coords,
    selected_cell: Coords,
    player: Player,
    exit: bool,
    possible_moves: Vec<Coords>,
}

impl App {
    pub fn new() -> Self {
        Self {
            grid: Board::new(),
            is_turn: 1,
            cursor_cell: Coords { x: 0, y: 0 },
            selected_cell: Coords { x: 0, y: 0 },
            exit: false,
            player: Player {
                id: 1,
                direction: 1,
            },
            possible_moves: vec![],
        }
    }
    fn next_turn(&mut self) {
        self.player.id = if self.is_turn == 1 { 2 } else { 1 };
        self.player.direction = if self.is_turn == 1 { -1 } else { 1 };
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
        if self.grid[self.cursor_cell].is_none() && self.possible_moves.contains(&self.cursor_cell)
        {
            self.grid[self.cursor_cell] = Some(Piece {
                piece_type: PieceType::Pawn,
                player_id: self.player.id,
            });
            self.grid[self.selected_cell] = None;
            self.next_turn();
            // if eating...
        }
        // selecting our own pawn
        if self.grid[self.cursor_cell].is_some() {
            //_and(|x| x.player == self.player_id) {
            self.selected_cell = self.cursor_cell;
            self.possible_moves = get_possible_moves(&self.grid, self.selected_cell, self.player);
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
        Paragraph::new(vec![
            Line::from(vec!["player 1: ".into(), "ciccio".green()]).left_aligned(),
            Line::from(vec!["player 2: ".into(), "pollo".red()]).left_aligned(),
            Line::from(format!("player {:?} is playing", self.is_turn)).right_aligned(),
        ])
        .render(info_area, buf);

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
                    color: if self.grid[coords].is_some_and(|x| x.player_id == self.player.id) {
                        Color::Green // player
                    } else {
                        Color::Red // opponent
                    },
                    radius: 5.0,
                };

                let cell_color = if coords == self.cursor_cell {
                    Color::LightGreen
                } else if coords == self.selected_cell {
                    Color::Yellow
                } else if self.possible_moves.contains(&coords) {
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
    ratatui::run(|terminal| App::new().run(terminal))
}
