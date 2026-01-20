use std::io;

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    layout::{
        Constraint::{self, Fill, Length, Min, Percentage, Ratio},
        Layout, Margin, Rect,
    },
    style::{Color, Stylize},
    text::{Line, Text},
    widgets::{Block, Padding, Paragraph, Widget},
};

static CELL_N: usize = 8;

fn main() -> io::Result<()> {
    ratatui::run(|terminal| App::default().run(terminal))
}

#[derive(Debug, Default)]
pub struct Grid {
    cells: Vec<usize>,
}

#[derive(Debug, Default)]
pub struct App {
    grid: Grid,
    is_turn: usize,
    cursor_cell: usize,
    selected_cell: usize,
    exit: bool,
}

impl App {
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

        let block = Block::bordered()
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

        // probably there's a cleaner way
        for (i, cell) in cells.enumerate() {
            let c = Paragraph::new(format!("{:02}", i)).block(Block::bordered());
            if i == self.cursor_cell {
                c.on_light_green().render(cell, buf);
            } else if i == self.selected_cell {
                c.on_yellow().render(cell, buf);
            } else {
                c.render(cell, buf);
            }
        }
    }
}
