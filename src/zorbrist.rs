use crate::position::{Position, Color, PieceType};
use rand::prelude::*;

#[derive(Clone, Debug)]
pub struct Zobrist {
    piece_square: [[u64; 64]; 12], // 6 pieces * 2 colors * 64 squares
    black_to_move: u64,
    castling_rights: [u64; 16],
    en_passant_file: [u64; 8],
}

impl Zobrist {
    pub fn new() -> Self {
        let mut rng = StdRng::seed_from_u64(0xDEADBEEF); // Fixed seed for reproducibility
        let mut z = Zobrist {
            piece_square: [[0; 64]; 12],
            black_to_move: rng.gen(),
            castling_rights: [0; 16],
            en_passant_file: [0; 8],
        };

        // Initialize piece-square values
        for piece_type in 0..12 {
            for square in 0..64 {
                z.piece_square[piece_type][square] = rng.gen();
            }
        }

        // Initialize castling rights values
        for i in 0..16 {
            z.castling_rights[i] = rng.gen();
        }

        // Initialize en passant file values
        for i in 0..8 {
            z.en_passant_file[i] = rng.gen();
        }

        z
    }

    pub fn hash_position(&self, pos: &Position) -> u64 {
        let mut hash = 0;

        // Hash pieces
        for piece in &pos.pieces {
            if piece.position == 0 {
                continue;
            }

            let square = piece.position.trailing_zeros() as usize;
            let piece_index = self.get_piece_index(piece.piece_type, piece.color);
            hash ^= self.piece_square[piece_index][square];
        }

        // Hash side to move
        if pos.active_color == Color::Black {
            hash ^= self.black_to_move;
        }

        // Hash castling rights
        let castling_index = pos.castling_rights.bits() as usize;
        hash ^= self.castling_rights[castling_index];

        // Hash en passant
        if let Some(ep_square) = pos.en_passant {
            let file = (ep_square.trailing_zeros() as usize) % 8;
            hash ^= self.en_passant_file[file];
        }

        hash
    }

    fn get_piece_index(&self, piece_type: PieceType, color: Color) -> usize {
        let base = match piece_type {
            PieceType::Pawn => 0,
            PieceType::Knight => 2,
            PieceType::Bishop => 4,
            PieceType::Rook => 6,
            PieceType::Queen => 8,
            PieceType::King => 10,
        };
        base + if color == Color::White { 0 } else { 1 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Game;

    #[test]
    fn test_same_position_same_hash() {
        let game = Game::new();
        let pos1 = Position::new(&game);
        let pos2 = Position::new(&game);
        let zobrist = Zobrist::new();

        assert_eq!(zobrist.hash_position(&pos1), zobrist.hash_position(&pos2));
    }

    #[test]
    fn test_different_positions_different_hash() {
        let game = Game::new();
        let pos1 = Position::new(&game);
        let mut pos2 = Position::new(&game);
        
        // Make a move in pos2
        let moves = pos2.get_all_legal_moves(&game);
        if !moves.is_empty() {
            pos2.make_move(moves[0]);
            let zobrist = Zobrist::new();
            assert_ne!(zobrist.hash_position(&pos1), zobrist.hash_position(&pos2));
        }
    }

    #[test]
    fn test_color_affects_hash() {
        let game = Game::new();
        let mut pos = Position::new(&game);
        let zobrist = Zobrist::new();
        let white_hash = zobrist.hash_position(&pos);
        
        pos.active_color = Color::Black;
        let black_hash = zobrist.hash_position(&pos);
        
        assert_ne!(white_hash, black_hash);
    }
}
