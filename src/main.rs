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

static CELL_N: usize = 8;
static PAWN_N: usize = 16;
static BOARD_SIZE: usize = CELL_N * CELL_N;

fn main() -> io::Result<()> {
    ratatui::run(|terminal| App::new().run(terminal))
}

#[derive(Debug, Default)]
pub struct App {
    grid: Vec<usize>,
    is_turn: usize,
    cursor_cell: usize,
    selected_cell: usize,
    player_id: usize,
    exit: bool,
}

impl App {
    pub fn new() -> Self {
        let mut grid = vec![0; CELL_N * CELL_N];
        for i in 0..PAWN_N {
            grid[i] = 1;
        }
        for j in (BOARD_SIZE - PAWN_N)..BOARD_SIZE {
            grid[j] = 2;
        }
        Self {
            grid,
            is_turn: 1,
            cursor_cell: 0,
            selected_cell: 0,
            exit: false,
            player_id: 2,
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
            _ => {}
        };
        Ok(())
    }

    // fn coords_to_index(x: usize, y: usize) -> usize {
    //     x * CELL_N + y
    // }

    fn exit(&mut self) {
        self.exit = true;
    }
    fn left(&mut self) {
        if self.cursor_cell != 0 {
            self.cursor_cell -= 1;
        }
    }
    fn down(&mut self) {
        if self.cursor_cell < CELL_N * (CELL_N - 1) {
            self.cursor_cell += CELL_N;
        }
    }
    fn up(&mut self) {
        if self.cursor_cell >= CELL_N {
            self.cursor_cell -= CELL_N;
        }
    }
    fn right(&mut self) {
        if self.cursor_cell != (CELL_N * CELL_N) - 1 {
            self.cursor_cell += 1;
        }
    }
    fn select(&mut self) {
        self.selected_cell = self.cursor_cell;
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
            Layout::vertical([Constraint::Percentage(5), Constraint::Percentage(95)]).spacing(1);
        let [info_area, board_area] = vertical_layout.areas(area.inner(Margin::new(1, 1)));

        // info area
        Paragraph::new(vec![
            Line::from(vec!["player: ".into(), "ciccio".green()]).left_aligned(),
            Line::from(vec!["opponent: ".into(), "pollo".red()]).left_aligned(),
        ])
        .render(info_area, buf);

        // board
        let cell_size = board_area.height / 8;
        let rows = Layout::vertical([Length(cell_size); 8])
            // .spacing(-1)
            .flex(ratatui::layout::Flex::Start)
            .split(board_area);

        let cells = rows.iter().flat_map(|row| {
            Layout::horizontal([Length(cell_size * 2); 8])
                // .spacing(-1)
                .flex(ratatui::layout::Flex::Center)
                .split(*row)
                .iter()
                .copied()
                .take(8)
                .collect::<Vec<Rect>>()
        });

        for (i, cell) in cells.enumerate() {
            let c = &Circle {
                x: 5.0,
                y: 5.0,
                color: if self.grid[i] == self.player_id {
                    Color::Green // player
                } else {
                    Color::Red // opponent
                },
                radius: 5.0,
            };
            let cell_color = if i == self.cursor_cell {
                Color::LightGreen
            } else if i == self.selected_cell {
                Color::Yellow
            } else {
                Color::Reset
            };

            Canvas::default()
                .block(Block::bordered().bg(cell_color))
                .marker(Marker::Braille)
                .x_bounds([0.0, 10.0])
                .y_bounds([0.0, 10.0])
                .paint(|ctx| {
                    if self.grid[i] != 0 {
                        ctx.draw(c);
                    }
                })
                .render(cell, buf);
        }
    }
}
