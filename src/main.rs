use std::{io, string};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    layout::{Constraint, Layout, Margin, Rect},
    style::Stylize,
    symbols::border,
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
            Layout::vertical([Constraint::Percentage(10), Constraint::Percentage(90)]).spacing(1);
        let [info_area, board_area] = vertical_layout.areas(area.inner(Margin::new(1, 1)));

        Paragraph::new(vec![
            Line::from(vec!["player: ".into(), "ciccio".green()]).left_aligned(),
            Line::from(vec!["opponent".into(), "pollo".red()]).left_aligned(),
        ])
        .render(info_area, buf);

        // let title = Line::from(" Counter App Tutorial ".bold());
        // let instructions = Line::from(vec![
        //     " Decrement ".into(),
        //     "<Left>".blue().bold(),
        //     " Increment ".into(),
        //     "<Right>".blue().bold(),
        //     " Quit ".into(),
        //     "<Q> ".blue().bold(),
        // ]);
        // let block = Block::bordered()
        //     .title(title.centered())
        //     .title_bottom(instructions.centered())
        //     .border_set(border::ROUNDED);

        // let counter_text = Text::from(vec![Line::from(vec![
        //     "Value: ".into(),
        //     self.counter.to_string().yellow(),
        // ])]);

        // Paragraph::new(counter_text)
        //     .centered()
        //     .block(block)
        //     .render(area, buf);
        // Paragraph::new(Text::from("ciao")).render(area, buf);
    }
}
