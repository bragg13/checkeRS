use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{
    board::Board,
    game_utils::Move,
    piece::{Piece, PieceType},
    player::{Player, PlayerId},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GameEvent {
    PlayerJoined { player: Player },
    TurnChanged { player_id: PlayerId },
    Move { mv: Move, player_id: PlayerId },
}

#[derive(Debug)]
pub struct GameState {
    pub grid: Board,
    pub is_turn: PlayerId,
    pub players: HashMap<PlayerId, Player>,
    history: Vec<GameEvent>,
}
impl GameState {
    pub fn new(players: Option<HashMap<PlayerId, Player>>) -> Self {
        Self {
            grid: Board::new(),
            is_turn: 1,
            players: players.unwrap_or(HashMap::new()),
            history: vec![],
        }
    }

    pub fn dispatch(&mut self, event: &GameEvent) -> Result<(), ()> {
        if !self.validate(event) {
            return Err(());
        }
        self.reduce(event);
        Ok(())
    }
    pub fn reduce(&mut self, event: &GameEvent) {
        match event {
            GameEvent::Move { mv, player_id } => self.move_pawn(mv, *player_id),
            GameEvent::TurnChanged { player_id } => self.is_turn = *player_id,
            GameEvent::PlayerJoined { .. } => {}
        }
        self.history.push(event.clone());
    }

    pub fn validate(&self, event: &GameEvent) -> bool {
        match event {
            GameEvent::PlayerJoined { player } => {
                if self.players.contains_key(&player.id) {
                    return false;
                }
            }
            GameEvent::Move { mv, player_id } => todo!(),
            GameEvent::TurnChanged { player_id } => todo!(),
        }
        true
    }

    fn move_pawn(&mut self, mv: &Move, player_id: PlayerId) {
        self.grid[mv.to()] = Some(Piece {
            piece_type: PieceType::Pawn,
            player_id,
        });

        // remove selected pawn from prev cell
        self.grid[mv.from()] = None;

        // eat if thats the case
        match mv {
            Move::Simple { .. } => {}
            Move::Capture { eat, .. } => {
                self.grid[*eat] = None;
                if let Some(player) = self.players.get_mut(&self.is_turn) {
                    player.score += 1;
                }
            }
        }
        //
    }
}
