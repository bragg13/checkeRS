use std::ops::{Index, IndexMut};

use crate::{
    CELL_N,
    coords::Coords,
    is_white,
    piece::{Piece, PieceType},
};

#[derive(Debug, Clone)]
pub struct Board(Vec<Vec<Option<Piece>>>);
impl Board {
    pub fn new() -> Self {
        let mut grid = vec![vec![None; CELL_N]; CELL_N];

        for i in 0..CELL_N {
            for j in 0..CELL_N {
                let coords = Coords { x: i, y: j };
                if i < 3 && is_white(coords) {
                    grid[i][j] = Some(Piece {
                        piece_type: PieceType::Pawn,
                        player: 2,
                    });
                } else if i > 4 && is_white(coords) {
                    grid[i][j] = Some(Piece {
                        piece_type: PieceType::Pawn,
                        player: 1,
                    });
                } else {
                    grid[i][j] = None;
                }
            }
        }

        Board(grid)
    }
}
impl Index<Coords> for Board {
    type Output = Option<Piece>;

    fn index(&self, index: Coords) -> &Self::Output {
        &self.0[index.y][index.x]
    }
}
impl IndexMut<Coords> for Board {
    fn index_mut(&mut self, index: Coords) -> &mut Self::Output {
        &mut self.0[index.y][index.x]
    }
}
