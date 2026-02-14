use std::{collections::HashMap, str::MatchIndices};

use crossterm::event::{Event, KeyCode, KeyEvent};
use ratatui::{
    layout::{Constraint, Flex, Layout, Margin, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Clear, Paragraph, Widget},
};
use store::{
    game_state::{ClientEvent, EndGameReason, GameEvent},
    player::{Player, PlayerId},
};
use tui_input::{Input, backend::crossterm::EventHandler};

#[derive(Debug)]
pub struct MainMenuScene {
    pub players: HashMap<PlayerId, Player>,
    submit: bool,
    username_in: Input,
    addr_in: Input,
    focused: usize,
    num_players: usize,
    prev_end_game_reason: Option<EndGameReason>,
}

impl Widget for &MainMenuScene {
    fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer) {
        let block = Block::bordered().title("Start a new game");
        let inner = block.inner(area.inner(Margin {
            horizontal: 1,
            vertical: 1,
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
                if self.submit {
                    Style::default().bg(Color::LightRed)
                } else {
                    Style::default().bg(Color::Yellow)
                }
            } else if self.can_submit() {
                Style::default().fg(Color::White)
            } else {
                Style::default().fg(Color::DarkGray)
            });

        button.render(chunks[2], buf);

        if self.submit && self.num_players < 2 {
            let block = Block::bordered().title("Alert");
            let popup_area = popup_area(area, 60, 20);
            let simple = throbber_widgets_tui::Throbber::default()
                .label("Waiting for another player to join...");
            Clear.render(popup_area, buf);
            block.render(popup_area, buf);
            simple.render(
                popup_area.inner(Margin {
                    horizontal: 3,
                    vertical: 3,
                }),
                buf,
            );
        } else if let Some(reason) = &self.prev_end_game_reason {
            let block = Block::bordered().title("Alert");
            let popup_area = popup_area(area, 60, 20);

            let simple = Paragraph::new(match reason {
                EndGameReason::PlayerLeft { player_id } => {
                    format!("You won the previous game because {player_id} left the game!")
                }
                EndGameReason::PlayerWon { winner } => {
                    format!("Player {winner} won the previous game!") // TODO: make this happen in Game and show username and score
                }
            });
            Clear.render(popup_area, buf);
            block.render(popup_area, buf);
            simple.render(
                popup_area.inner(Margin {
                    horizontal: 3,
                    vertical: 3,
                }),
                buf,
            );
        }
    }
}

impl MainMenuScene {
    pub fn new(prev_end_game_reason: Option<EndGameReason>) -> Self {
        Self {
            submit: false,
            username_in: Input::default().with_value("andrea".into()),
            addr_in: Input::default().with_value("127.0.0.1:5000".into()),
            focused: 2,
            num_players: 0,
            players: HashMap::new(),
            prev_end_game_reason: prev_end_game_reason,
        }
    }
    fn can_submit(&self) -> bool {
        !self.username_in.value().is_empty() && !self.addr_in.value().is_empty()
    }

    pub fn handle_server_events(&mut self, game_event: GameEvent) -> Option<ClientEvent> {
        match game_event {
            GameEvent::PlayerJoined { player } => {
                self.players.insert(player.id, player);
                return None;
            }
            GameEvent::TurnChanged { player_id } => {
                Some(ClientEvent::GoToGame(self.players.clone(), player_id))
            }
            _ => None,
        }
    }

    pub fn handle_input(&mut self, key_event: KeyEvent) -> Option<ClientEvent> {
        match key_event.code {
            KeyCode::Enter => {
                if self.can_submit() {
                    self.submit = true;
                    Some(ClientEvent::GoToLobby(
                        String::from(self.username_in.value()),
                        String::from(self.addr_in.value()),
                    ))
                } else {
                    return None;
                }
            }
            KeyCode::Down => {
                if self.focused < 2 {
                    self.focused += 1
                }
                return None;
            }
            KeyCode::Up => {
                if self.focused > 0 {
                    self.focused -= 1
                }
                return None;
            }
            _ => match self.focused {
                0 => {
                    self.username_in.handle_event(&Event::Key(key_event));
                    return None;
                }
                1 => {
                    self.addr_in.handle_event(&Event::Key(key_event));
                    return None;
                }
                _ => return None,
            },
        }
    }
}

/// helper function to create a centered rect using up certain percentage of the available rect `r`
fn popup_area(area: Rect, percent_x: u16, percent_y: u16) -> Rect {
    let vertical = Layout::vertical([Constraint::Percentage(percent_y)]).flex(Flex::Center);
    let horizontal = Layout::horizontal([Constraint::Percentage(percent_x)]).flex(Flex::Center);
    let [area] = vertical.areas(area);
    let [area] = horizontal.areas(area);
    area
}
