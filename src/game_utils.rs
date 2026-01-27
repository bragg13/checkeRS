use crate::{CELL_N, board::Board, coords::Coords, player::Player};

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
    cell: Coords,
    player: &Player,
) -> (Vec<Coords>, Vec<Coords>) {
    let mut empty = vec![];
    let mut edible = vec![];
    // let offset_y = if player.direction == 1 { -1 } else { 1 };
    cell.diag()
        .into_iter()
        .for_each(|diag_coord| match grid[diag_coord] {
            None => {
                if player.direction == 1 {
                    // moves toward up (+1)
                    if diag_coord.y < cell.y {
                        empty.push(diag_coord);
                    }
                } else {
                    // move towards down (-1)
                    if diag_coord.y > cell.y {
                        empty.push(diag_coord);
                    }
                }
            }
            Some(next_cell) => {
                if next_cell.player_id != player.id {
                    if player.direction == 1 {
                        // look towards up (+1)
                        if diag_coord.x > cell.x {
                            // this diagonal cell is on the right
                            let landing_coords = Coords {
                                x: diag_coord.x + 1,
                                y: diag_coord.y - 1,
                            };
                            match grid[landing_coords] {
                                Some(_) => {}
                                None => edible.push(landing_coords),
                            }
                        } else if diag_coord.x < cell.x {
                            // this diagonal cell is on the LEFT
                            let landing_coords = Coords {
                                x: diag_coord.x - 1,
                                y: diag_coord.y - 1,
                            };
                            match grid[landing_coords] {
                                Some(_) => {}
                                None => edible.push(landing_coords),
                            }
                        }
                    } else {
                        // move towards down (-1)
                        if diag_coord.x > cell.x {
                            // this diagonal cell is on the right
                            let landing_coords = Coords {
                                x: diag_coord.x + 1,
                                y: diag_coord.y + 1,
                            };
                            match grid[landing_coords] {
                                Some(_) => {}
                                None => edible.push(landing_coords),
                            }
                        } else if diag_coord.x < cell.x {
                            // this diagonal cell is on the LEFT
                            let landing_coords = Coords {
                                x: diag_coord.x - 1,
                                y: diag_coord.y + 1,
                            };
                            match grid[landing_coords] {
                                Some(_) => {}
                                None => edible.push(landing_coords),
                            }
                        }
                    }
                }
            }
        });
    (empty, edible)
}
