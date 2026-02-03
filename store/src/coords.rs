use std::ops::{Add, Sub};

use crate::CELL_N;

#[derive(Debug, Default, Copy, Clone, PartialEq)]
pub struct Coords {
    pub x: usize,
    pub y: usize,
}
impl Add for Coords {
    type Output = Coords;

    fn add(self, rhs: Self) -> Self::Output {
        Coords {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}
impl Sub for Coords {
    type Output = (i32, i32);

    fn sub(self, rhs: Self) -> Self::Output {
        (
            (self.x as i32 - rhs.x as i32),
            (self.y as i32 - rhs.y as i32),
        )
    }
}
impl Coords {
    // very verbose and not scalable, but for a simple game with only 4 diagonals it is fine.
    // this lets me avoid many casting or using checked_sub as im working with usize and i32
    // and arithmetic is annoying
    pub fn diag(self) -> Vec<Coords> {
        let mut v = Vec::new();

        // Top-left
        if self.x > 0 && self.y > 0 {
            v.push(Coords {
                x: self.x - 1,
                y: self.y - 1,
            });
        }
        // Top-right
        if self.x < CELL_N - 1 && self.y > 0 {
            v.push(Coords {
                x: self.x + 1,
                y: self.y - 1,
            });
        }
        // Bottom-left
        if self.x > 0 && self.y < CELL_N - 1 {
            v.push(Coords {
                x: self.x - 1,
                y: self.y + 1,
            });
        }
        // Bottom-right
        if self.x < CELL_N - 1 && self.y < CELL_N - 1 {
            v.push(Coords {
                x: self.x + 1,
                y: self.y + 1,
            });
        }

        v
    }
}
