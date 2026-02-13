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
    PlayerLeft { player_id: PlayerId },
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
    pub fn new(players: HashMap<PlayerId, Player>, starting_turn: PlayerId) -> Self {
        Self {
            grid: Board::new(&players, starting_turn),
            is_turn: starting_turn,
            players: players,
            history: vec![],
        }
    }
    pub fn next_turn(&mut self) -> PlayerId {
        // TODO: mi piace usare * ?
        let next_player = self
            .players
            .keys()
            .filter(|id| **id != self.is_turn)
            .next()
            .unwrap();
        self.is_turn = *next_player;
        *next_player
    }

    pub fn dispatch(&mut self, event: &GameEvent) -> Result<(), String> {
        self.validate(event)?;
        self.reduce(event)?;
        Ok(())
    }
    pub fn reduce(&mut self, event: &GameEvent) -> Result<(), String> {
        match event {
            GameEvent::Move { mv, player_id } => {
                self.move_pawn(mv, *player_id)?;
            }
            GameEvent::TurnChanged { player_id } => {
                self.is_turn = *player_id;
            }
            GameEvent::PlayerJoined { .. } => {}
            GameEvent::PlayerLeft { player_id } => {
                self.players.remove(player_id).unwrap();
                ()
            }
        }
        self.history.push(event.clone());
        Ok(())
    }

    pub fn validate(&self, event: &GameEvent) -> Result<(), String> {
        match event {
            GameEvent::PlayerJoined { player } => {
                if self.players.contains_key(&player.id) {
                    return Err(format!(
                        "Players list already contains this id: {}",
                        player.id
                    ));
                }
            }
            GameEvent::Move { player_id, .. } => {
                if self.is_turn != *player_id {
                    return Err(format!("Not your turn, player {}", player_id));
                }
            }
            GameEvent::PlayerLeft { player_id } => {
                if !self.players.contains_key(&player_id) {
                    return Err(format!("Player is not playing: {}", player_id));
                }
            }
            GameEvent::TurnChanged { player_id } => {
                if self.is_turn == *player_id {
                    return Err(format!("Player {player_id} is already playing"));
                }
            }
        }
        Ok(())
    }

    // TODO: catch errors
    fn move_pawn(&mut self, mv: &Move, player_id: PlayerId) -> Result<(), String> {
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
        Ok(())
    }
}
