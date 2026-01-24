use crate::{CELL_N, board::Board, coords::Coords};

fn _index_to_coords(i: usize) -> (usize, usize) {
    (i / CELL_N, i % CELL_N)
}
pub fn coords_to_index(coords: Coords) -> usize {
    coords.y * CELL_N + coords.x
}
pub fn is_white(coords: Coords) -> bool {
    (coords.x + coords.y) % 2 == 0
}

pub fn get_possible_moves(grid: &Board, cell: Coords, player: usize) -> Vec<Coords> {
    let mut empty = vec![];
    cell.diag()
        .into_iter()
        .for_each(|diag_coord| match grid[diag_coord] {
            None => {
                // if empty cell and above us, we can move
                // TODO: take into account player side
                if diag_coord.y < cell.y {
                    empty.push(diag_coord);
                }
            }
            Some(next_cell) => {
                if next_cell.player != player {
                    // maybe we can eat
                    let _direction = diag_coord - cell;
                    // if direction.
                }
            }
        });
    empty
}
