use cli_log::info;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{layout::Constraint::Length, style::Stylize, widgets::BorderType};
use std::collections::HashMap;

use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Margin, Rect},
    style::Color,
    symbols::Marker,
    widgets::{
        Block, Paragraph, Widget,
        canvas::{Canvas, Circle},
    },
};
use store::{
    CELL_N,
    coords::Coords,
    game_state::{ClientEvent, GameEvent, GameState},
    game_utils::{Move, coords_to_index, get_possible_moves, is_white},
    player::{Player, PlayerId},
};

#[derive(Debug)]
pub struct GameScene {
    game_state: GameState,
    possible_moves: Vec<Move>,
    cursor_cell: Coords,
    selected_cell: Option<Coords>,
    player_id: PlayerId,
}

impl GameScene {
    pub fn new(
        players: HashMap<PlayerId, Player>,
        player_id: PlayerId,
        starting_player: PlayerId,
    ) -> Self {
        let player = players.get(&player_id).unwrap();
        Self {
            game_state: GameState::new(players.clone(), starting_player),
            cursor_cell: Coords {
                x: 0,
                y: if player.direction == -1 { 0 } else { 7 },
            },
            selected_cell: None,
            player_id,
            possible_moves: vec![],
        }
    }
    pub fn handle_input(&mut self, key_event: KeyEvent) -> Option<ClientEvent> {
        if self.game_state.players.len() < 2 {
            return None;
        }
        if key_event.code == KeyCode::Char(' ') && self.game_state.is_turn == self.player_id {
            return self.select();
        } else {
            match key_event.code {
                KeyCode::Left => self.left(),
                KeyCode::Down => self.down(),
                KeyCode::Up => self.up(),
                KeyCode::Right => self.right(),
                _ => {}
            }
            return None;
        }
    }
    pub fn handle_server_events(&mut self, game_event: GameEvent) -> Option<ClientEvent> {
        self.possible_moves.clear();
        self.selected_cell = None;
        match self.game_state.reduce(&game_event) {
            Ok(client_event) => return client_event,
            Err(err) => {
                info!("❌ Error while reducing game event: {err}");
                None
            }
        }
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
    fn select(&mut self) -> Option<ClientEvent> {
        if !is_white(self.cursor_cell) {
            return None;
        }

        // selecting empty cell
        if self.game_state.grid[self.cursor_cell].is_none() {
            let selected_move = self
                .possible_moves
                .iter()
                .find(|possible_move| possible_move.to() == self.cursor_cell);
            match selected_move {
                Some(_mv) => {
                    return Some(ClientEvent::SendToServer(GameEvent::Move {
                        mv: selected_move.unwrap().clone(), // TODO
                        player_id: self.player_id,
                    }));
                }
                None => {}
            }
        }

        // selecting our own pawn
        if self.game_state.grid[self.cursor_cell].is_some_and(|x| x.player_id == self.player_id) {
            self.selected_cell = Some(self.cursor_cell);
            if let Some(selected_cell) = self.selected_cell
                && let Some(player) = self.game_state.players.get(&self.player_id)
            {
                match get_possible_moves(&self.game_state.grid, selected_cell, player) {
                    Ok(moves) => self.possible_moves = moves,
                    Err(err) => info!("❌ Error while selecting own's pawn: {err}"),
                }
            } else {
                info!("❌ Could now unwrap `selected_cell` or `player`");
            }
        }
        return None;
    }
}

impl Widget for &GameScene {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let vertical_layout =
            Layout::vertical([Constraint::Percentage(8), Constraint::Percentage(92)]).spacing(1);
        let [info_area, board_area] = vertical_layout.areas(area.inner(Margin::new(1, 1)));

        // info area - TODO: i wanna see first the client name
        let mut players_scoreboard = vec![];
        for player in self.game_state.players.iter() {
            players_scoreboard.push(
                player
                    .1
                    .pretty_print_scoreboard(
                        self.game_state.is_turn,
                        if player.1.id == self.player_id {
                            Color::Green
                        } else {
                            Color::Red
                        },
                    )
                    .left_aligned(),
            );
        }
        Paragraph::new(players_scoreboard).render(info_area, buf);

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
                    color: if self.game_state.grid[coords]
                        .is_some_and(|x| x.player_id == self.player_id)
                    {
                        Color::Green // player
                    } else {
                        Color::Red // opponent
                    },
                    radius: 5.0,
                };
                let bg_color = if is_white(coords) {
                    Color::White
                } else {
                    Color::Black
                };

                let border_color = if coords == self.cursor_cell {
                    if self.game_state.is_turn == self.player_id {
                        Color::LightGreen
                    } else {
                        Color::Gray
                    }
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
                    bg_color
                };

                Canvas::default()
                    .block(
                        Block::bordered()
                            .bg(bg_color)
                            .fg(border_color)
                            .border_type(BorderType::Double),
                    )
                    .marker(Marker::Braille)
                    .background_color(bg_color)
                    .x_bounds([0.0, 10.0])
                    .y_bounds([0.0, 10.0])
                    .paint(|ctx| {
                        if self.game_state.grid[coords].is_some_and(|x| x.player_id > 0) {
                            ctx.draw(c);
                        }
                    })
                    .render(cells[coords_to_index(coords)], buf);
            }
        }
    }
}
