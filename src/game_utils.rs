use cli_log::debug;

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

pub fn get_possible_moves(grid: &Board, cell: Coords, player: Player) -> Vec<Coords> {
    let mut empty = vec![];
    cell.diag()
        .into_iter()
        .for_each(|diag_coord| match grid[diag_coord] {
            None => {
                // debug!("{:?}", player.direction);
                if player.direction == 1 {
                    if diag_coord.y < cell.y {
                        empty.push(diag_coord);
                    }
                } else {
                    if diag_coord.y > cell.y {
                        empty.push(diag_coord);
                    }
                }
            }
            Some(next_cell) => {
                if next_cell.player_id != player.id {
                    // maybe we can eat
                    let _direction = diag_coord - cell;
                    // if direction.
                }
            }
        });
    empty
}
