use std::collections::HashMap;

use crate::{
    board::Board,
    player::{Player, PlayerId},
};

#[derive(Debug, Clone)]
pub enum GameEvent {
    PlayerJoined { player: Player },
}

#[derive(Debug)]
pub struct GameState {
    pub grid: Board,
    pub is_turn: PlayerId,
    pub players: HashMap<PlayerId, Player>,
    history: Vec<GameEvent>,
}
impl GameState {
    pub fn new() -> Self {
        // let mut players = HashMap::new();
        // players.insert(
        //     1 as PlayerId,
        //     Player {
        //         direction: 1,
        //         id: 1 as PlayerId,
        //         name: "Kasparov".to_string(),
        //         score: 0,
        //     },
        // );
        // players.insert(
        //     2 as PlayerId,
        //     Player {
        //         direction: -1,
        //         id: 2 as PlayerId,
        //         name: "Magnussen".to_string(),
        //         score: 0,
        //     },
        // );
        Self {
            grid: Board::new(),
            is_turn: 1,
            players: HashMap::new(), //players,
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
            GameEvent::PlayerJoined { player } => {
                println!("player {:?} joined", player.id);
                self.players.insert(player.id, player.clone());
            }
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
        }
        true
    }
}
