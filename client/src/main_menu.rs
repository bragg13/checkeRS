use cli_log::info;
use crossterm::event::{Event, KeyCode, KeyEvent};
use ratatui::{
    layout::{Constraint, Layout, Margin, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph, Widget},
};
use store::player::{Player, PlayerId};
use tui_input::{Input, backend::crossterm::EventHandler};

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

        let username_block = Block::default()
            .title("Username")
            .borders(Borders::ALL)
            .border_style(if self.focused == 0 {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default()
            });

        Paragraph::new(self.username_in.value()).render(username_block.inner(chunks[0]), buf);
        username_block.render(chunks[0], buf);

        let addr_block = Block::default()
            .title("Address")
            .borders(Borders::ALL)
            .border_style(if self.focused == 1 {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default()
            });
        Paragraph::new(self.addr_in.value()).render(addr_block.inner(chunks[1]), buf);
        addr_block.render(chunks[1], buf);

        let button = Paragraph::new("[ Connect ]")
            .centered()
            .block(Block::default().borders(Borders::ALL))
            .style(if self.focused == 2 {
                Style::default().bg(Color::Yellow)
            } else if self.can_submit() {
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
            username_in: Input::default().with_value("andrea".into()),
            addr_in: Input::default().with_value("127.0.0.1:5000".into()),
            focused: 2,
        }
    }
    fn can_submit(&self) -> bool {
        !self.username_in.value().is_empty() && !self.addr_in.value().is_empty()
    }

    pub fn handle_input(&mut self, key_event: KeyEvent) -> SceneTransition {
        match key_event.code {
            KeyCode::Enter => {
                if self.can_submit() && self.focused == 2 {
                    self.submit = true;
                    SceneTransition::ToGame(
                        String::from(self.username_in.value()),
                        String::from(self.addr_in.value()),
                    )
                } else {
                    SceneTransition::None
                }
            }
            KeyCode::Down => {
                if self.focused < 2 {
                    self.focused += 1
                }
                SceneTransition::None
            }
            KeyCode::Up => {
                if self.focused > 0 {
                    self.focused -= 1
                }
                SceneTransition::None
            }
            _ => match self.focused {
                0 => {
                    self.username_in.handle_event(&Event::Key(key_event));
                    SceneTransition::None
                }
                1 => {
                    self.addr_in.handle_event(&Event::Key(key_event));
                    SceneTransition::None
                }
                _ => SceneTransition::None,
            },
        }
    }
}
