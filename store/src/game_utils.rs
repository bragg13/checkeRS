use serde::{Deserialize, Serialize};

use crate::{CELL_N, board::Board, coords::Coords, player::Player};

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub enum Move {
    Simple {
        from: Coords,
        to: Coords,
    },
    Capture {
        from: Coords,
        to: Coords,
        eat: Coords,
    },
}
impl Move {
    pub fn to(&self) -> Coords {
        match self {
            Move::Capture { to, .. } => *to,
            Move::Simple { to, .. } => *to,
        }
    }
    pub fn from(&self) -> Coords {
        match self {
            Move::Capture { from, .. } => *from,
            Move::Simple { from, .. } => *from,
        }
    }
}

fn _index_to_coords(i: usize) -> (usize, usize) {
    (i / CELL_N, i % CELL_N)
}
pub fn coords_to_index(coords: Coords) -> usize {
    coords.y * CELL_N + coords.x
}
pub fn is_white(coords: Coords) -> bool {
    (coords.x + coords.y) % 2 == 0
}

pub fn get_possible_moves(
    grid: &Board,
    original_cell: Coords,
    player: &Player,
) -> Result<Vec<Move>, String> {
    let mut moves = vec![];

    // check edible moves
    original_cell
        .diag()
        .into_iter()
        .filter(|cell| grid[*cell].is_some_and(|c| c.player_id != player.id))
        .for_each(|edible_coords| {
            let landing_coords = if player.direction == 1 {
                // look towards up (+1)
                if edible_coords.x > original_cell.x {
                    if edible_coords.x < CELL_N - 1 && edible_coords.y > 0 {
                        // this diagonal original_cell is on the right
                        Some(Coords {
                            x: edible_coords.x + 1,
                            y: edible_coords.y - 1,
                        })
                    } else {
                        None
                    }
                } else if edible_coords.x < original_cell.x {
                    if edible_coords.x > 0 && edible_coords.y > 0 {
                        Some(Coords {
                            x: edible_coords.x - 1,
                            y: edible_coords.y - 1,
                        })
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                // move towards down (-1)
                if edible_coords.x > original_cell.x {
                    // this diagonal original_cell is on the right
                    if edible_coords.x < CELL_N - 1 && edible_coords.y < CELL_N - 1 {
                        Some(Coords {
                            x: edible_coords.x + 1,
                            y: edible_coords.y + 1,
                        })
                    } else {
                        None
                    }
                } else if edible_coords.x < original_cell.x {
                    // this diagonal original_cell is on the LEFT
                    if edible_coords.x > 0 && edible_coords.y < CELL_N - 1 {
                        Some(Coords {
                            x: edible_coords.x - 1,
                            y: edible_coords.y + 1,
                        })
                    } else {
                        None
                    }
                } else {
                    None
                }
            };
            if let Some(landing) = landing_coords {
                if grid[landing].is_none() {
                    moves.push(Move::Capture {
                        from: original_cell,
                        to: landing,
                        eat: edible_coords,
                    })
                }
            }
        });

    // rule: forced to capture if can capture
    if !moves.is_empty() {
        return Ok(moves);
    }

    original_cell
        .diag()
        .into_iter()
        .filter(|cell| grid[*cell].is_none())
        .for_each(|empty_coords| {
            if player.direction == 1 {
                // moves toward up (+1)
                if empty_coords.y < original_cell.y {
                    moves.push(Move::Simple {
                        from: original_cell,
                        to: empty_coords,
                    });
                }
            } else {
                // move towards down (-1)
                if empty_coords.y > original_cell.y {
                    moves.push(Move::Simple {
                        from: original_cell,
                        to: empty_coords,
                    });
                }
            }
        });
    Ok(moves)
}
