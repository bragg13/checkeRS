#[derive(Debug, Clone, Copy)]
pub enum PieceType {
    Pawn,
    King,
}

#[derive(Debug, Clone, Copy)]
pub struct Piece {
    pub piece_type: PieceType,
    pub player: usize,
}
impl Piece {
    pub fn new(piece_type: PieceType, player: usize) -> Self {
        Piece {
            piece_type: piece_type,
            player: player,
        }
    }
}
