#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Move {
    from: Square,
    to: Square,
    promotion: Option<PieceType>,
    is_capture: bool,
    is_castle: bool,
    is_en_passant: bool,
}
