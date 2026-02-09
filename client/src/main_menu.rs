use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Constraint, Layout, Margin, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph, Widget},
};
use tui_input::Input;

use crate::{Scene, SceneTransition, game::GameScene};
#[derive(Debug)]
pub struct MainMenuScene {
    submit: bool,
    username_in: Input,
    addr_in: Input,
    focused: usize,
}

impl Widget for &MainMenuScene {
    fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer) {
        let block = Block::bordered().title("Start a new game");
        let inner = block.inner(area.inner(Margin {
            horizontal: 10,
            vertical: 10,
        }));
        block.render(area, buf);

        let chunks = Layout::vertical([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
        ])
        .split(inner);

        let username = Block::default()
            .title("Username")
            .borders(Borders::ALL)
            .border_style(if self.focused == 0 {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default()
            });

        username.render(chunks[0], buf);

        let addr = Block::default()
            .title("Address")
            .borders(Borders::ALL)
            .border_style(if self.focused == 1 {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default()
            });
        addr.render(chunks[1], buf);

        let button = Paragraph::new("[ Connect ]")
            .centered()
            .block(Block::default().borders(Borders::ALL))
            .style(if self.focused == 2 {
                Style::default().bg(Color::Yellow)
            } else if self.submit {
                Style::default().fg(Color::White)
            } else {
                Style::default().fg(Color::DarkGray)
            });

        button.render(chunks[2], buf);
    }
}

impl MainMenuScene {
    pub fn new() -> Self {
        Self {
            submit: false,
            username_in: Input::default(),
            addr_in: Input::default(),
            focused: 0,
        }
    }
    pub fn handle_input(&mut self, key_event: KeyEvent) -> SceneTransition {
        match key_event.code {
            KeyCode::Enter => {
                self.submit = true;
                SceneTransition::ToGame
            }
            KeyCode::Char('j') => {
                if self.focused < 2 {
                    self.focused += 1
                }
                SceneTransition::None
            }
            KeyCode::Char('k') => {
                if self.focused > 0 {
                    self.focused -= 1
                }
                SceneTransition::None
            }
            _ => SceneTransition::None,
        }
    }
}
