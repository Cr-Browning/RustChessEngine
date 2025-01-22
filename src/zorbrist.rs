pub struct Zobrist {
    piece_square: [[u64; 64]; 12], // 6 pieces * 2 colors * 64 squares
    black_to_move: u64,
    castling_rights: [u64; 16],
    en_passant_file: [u64; 8],
}
