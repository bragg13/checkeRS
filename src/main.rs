use std::{
    io,
    ops::{Add, Index, IndexMut, Sub},
};

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

static CELL_N: usize = 8;

fn main() -> io::Result<()> {
    ratatui::run(|terminal| App::new().run(terminal))
}

#[derive(Debug, Clone, Copy)]
pub enum PieceType {
    Pawn,
    King,
}

#[derive(Debug, Default, Copy, Clone, PartialEq)]
pub struct Coords {
    pub x: usize,
    pub y: usize,
}
impl Add for Coords {
    type Output = Coords;

    fn add(self, rhs: Self) -> Self::Output {
        Coords {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}
impl Sub for Coords {
    type Output = (i32, i32);

    fn sub(self, rhs: Self) -> Self::Output {
        (
            (self.x as i32 - rhs.x as i32),
            (self.y as i32 - rhs.y as i32),
        )
    }
}
impl Coords {
    // very verbose and not scalable, but for a simple game with only 4 diagonals it is fine.
    // this lets me avoid many casting or using checked_sub as im working with usize and i32
    // and arithmetic is annoying
    pub fn diag(self) -> Vec<Coords> {
        let mut v = Vec::new();

        // Top-left
        if self.x > 0 && self.y > 0 {
            v.push(Coords {
                x: self.x - 1,
                y: self.y - 1,
            });
        }
        // Top-right
        if self.x < CELL_N - 1 && self.y > 0 {
            v.push(Coords {
                x: self.x + 1,
                y: self.y - 1,
            });
        }
        // Bottom-left
        if self.x > 0 && self.y < CELL_N - 1 {
            v.push(Coords {
                x: self.x - 1,
                y: self.y + 1,
            });
        }
        // Bottom-right
        if self.x < CELL_N - 1 && self.y < CELL_N - 1 {
            v.push(Coords {
                x: self.x + 1,
                y: self.y + 1,
            });
        }

        v
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Piece {
    pub piece_type: PieceType,
    pub player: usize,
}
impl Piece {
    pub fn new(piece_type: PieceType, player: usize) -> Self {
        Piece {
            piece_type: piece_type,
            player: player,
        }
    }
}
#[derive(Debug, Clone)]
pub struct Board(Vec<Vec<Option<Piece>>>);
impl Board {
    pub fn new() -> Self {
        let mut grid = vec![vec![None; CELL_N]; CELL_N];

        for i in 0..CELL_N {
            for j in 0..CELL_N {
                let coords = Coords { x: i, y: j };
                if i < 3 && is_white(coords) {
                    grid[i][j] = Some(Piece {
                        piece_type: PieceType::Pawn,
                        player: 2,
                    });
                } else if i > 4 && is_white(coords) {
                    grid[i][j] = Some(Piece {
                        piece_type: PieceType::Pawn,
                        player: 1,
                    });
                } else {
                    grid[i][j] = None;
                }
            }
        }

        Board(grid)
    }
}
impl Index<Coords> for Board {
    type Output = Option<Piece>;

    fn index(&self, index: Coords) -> &Self::Output {
        &self.0[index.y][index.x]
    }
}
impl IndexMut<Coords> for Board {
    fn index_mut(&mut self, index: Coords) -> &mut Self::Output {
        &mut self.0[index.y][index.x]
    }
}

#[derive(Debug)]
pub struct App {
    grid: Board,
    is_turn: usize,
    cursor_cell: Coords,
    selected_cell: Coords,
    player_id: usize,
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
            player_id: 1,
            possible_moves: vec![],
        }
    }
    fn next_turn(&mut self) {
        // random move from opponent
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
                player: self.player_id,
            });
            self.grid[self.selected_cell] = None;
            self.next_turn();
            // if eating...
        }
        // selecting our own pawn
        if self.grid[self.cursor_cell].is_some() {
            //_and(|x| x.player == self.player_id) {
            self.selected_cell = self.cursor_cell;
            self.possible_moves =
                get_possible_moves(&self.grid, self.selected_cell, self.player_id);
        }
    }
}
// HELPER FUNCTIONS to put into a separate crate probably
fn _index_to_coords(i: usize) -> (usize, usize) {
    (i / CELL_N, i % CELL_N)
}
fn coords_to_index(coords: Coords) -> usize {
    coords.y * CELL_N + coords.x
}
pub fn is_white(coords: Coords) -> bool {
    (coords.x + coords.y) % 2 == 0
}

pub fn get_possible_moves(grid: &Board, cell: Coords, player: usize) -> Vec<Coords> {
    // can move diagonally forward
    let mut empty = vec![];
    cell.diag()
        .into_iter()
        .for_each(|diag_coord| match grid[diag_coord] {
            None => {
                // if empty cell and above us, we can move
                // TODO: take into account player side
                if diag_coord.y < cell.y {
                    empty.push(diag_coord);
                }
            }
            Some(next_cell) => {
                if next_cell.player != player {
                    // maybe we can eat
                    let _direction = diag_coord - cell;
                    // if direction.
                }
            }
        });
    empty
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
            // .spacing(-1)
            .flex(ratatui::layout::Flex::Start)
            .split(board_area);

        let cells = rows
            .iter()
            .flat_map(|row| {
                Layout::horizontal([Length(cell_size * 2); 8])
                    // .spacing(-1)
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
                    color: if self.grid[coords].is_some_and(|x| x.player == self.player_id) {
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
                        if self.grid[coords].is_some_and(|x| x.player != 0) {
                            ctx.draw(c);
                        }
                    })
                    .render(cells[coords_to_index(coords)], buf);
            }
        }
    }
}
