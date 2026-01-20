use std::{cell, io};

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    layout::{
        Constraint::{self, Fill, Length, Min, Percentage, Ratio},
        Layout, Margin, Rect,
    },
    style::Stylize,
    text::{Line, Text},
    widgets::{Block, Padding, Paragraph, Widget},
};

fn main() -> io::Result<()> {
    ratatui::run(|terminal| App::default().run(terminal))
}

#[derive(Debug, Default)]
pub struct App {
    // player_name: string,
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
                    // KeyCode::Left => self.decrement_counter(),
                    // KeyCode::Right => self.increment_counter(),
                    _ => {}
                }
            }
            _ => {}
        };
        Ok(())
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    // fn increment_counter(&mut self) {
    //     self.counter += 1;
    // }

    // fn decrement_counter(&mut self) {
    //     self.counter -= 1;
    // }
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
            .flex(ratatui::layout::Flex::Start)
            .split(board_area);

        let cells = rows.iter().flat_map(|row| {
            Layout::horizontal([Length(cell_size * 2); 8])
                .flex(ratatui::layout::Flex::Center)
                .split(*row)
                .iter()
                .copied()
                .take(8)
                .collect::<Vec<Rect>>()
        });

        for (i, cell) in cells.enumerate() {
            Paragraph::new(format!("{:02}", i))
                .block(Block::bordered())
                .render(cell, buf);
        }
    }
}
