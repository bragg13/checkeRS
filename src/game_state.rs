use std::collections::HashMap;

use crate::{
    board::Board,
    player::{Player, PlayerId},
};

#[derive(Debug)]
pub struct GameState {
    pub grid: Board,
    pub is_turn: PlayerId,
    pub players: HashMap<PlayerId, Player>,
    history: Vec<GameEvent>,
}

#[derive(Debug)]
pub enum GameEvent {
    PlayerJoined { player: Player },
}
