#[derive(Debug, Clone, Copy)]
pub enum PieceType {
    Pawn,
    King,
}

#[derive(Debug, Clone, Copy)]
pub struct Piece {
    pub piece_type: PieceType,
    pub player_id: u64,
}
impl Piece {
    pub fn new(piece_type: PieceType, player: u64) -> Self {
        Piece {
            piece_type: piece_type,
            player_id: player,
        }
    }
}
