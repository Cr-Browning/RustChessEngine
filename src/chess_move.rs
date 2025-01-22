use crate::position::{Square, PieceType};

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct Move {
    from: Square,
    to: Square,
    promotion: Option<PieceType>,
    is_capture: bool,
    is_castle: bool,
    is_en_passant: bool,
}

