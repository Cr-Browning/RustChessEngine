use crate::position::{Position, PieceType, Color};
use crate::Game;
use crate::utils::{bit_scan, extract_bits};
use crate::evaluation::Evaluation;

// Move scoring constants
const CAPTURE_SCORE_BASE: i32 = 10000;
const PIECE_VALUES: [i32; 6] = [
    100,   // Pawn
    500,   // Rook
    320,   // Knight
    330,   // Bishop
    900,   // Queen
    0,     // King (not used for captures)
];

#[derive(Clone)]
pub struct MoveOrderer {
    move_scores: Vec<(u64, i32)>, // (move, score) pairs
}

impl MoveOrderer {
    pub fn new() -> Self {
        MoveOrderer {
            move_scores: Vec::new(),
        }
    }

    // Score and sort moves based on various heuristics
    pub fn order_moves(&mut self, position: &Position, moves: &[u64], game: &Game) -> Vec<u64> {
        self.move_scores.clear();
        
        // Score each move
        for &mov in moves {
            let score = self.score_move(position, mov, game);
            self.move_scores.push((mov, score));
        }

        // Sort moves by score in descending order
        self.move_scores.sort_by(|a, b| b.1.cmp(&a.1));

        // Return sorted moves
        self.move_scores.iter().map(|(mov, _)| *mov).collect()
    }

    fn score_move(&self, position: &Position, mov: u64, _game: &Game) -> i32 {
        let mut score = 0;

        // Get the moving piece and target square
        let from_square = 1u64 << (mov & 0x3F);  // Lower 6 bits for from square
        let to_square = 1u64 << ((mov >> 6) & 0x3F);  // Next 6 bits for to square
        
        // Score captures
        if let Some(captured_piece) = position.get_piece_at(to_square) {
            if let Some(attacker_type) = position.get_piece_type_at(from_square) {
                score += CAPTURE_SCORE_BASE + 
                        PIECE_VALUES[captured_piece as usize] -
                        (PIECE_VALUES[attacker_type as usize] / 100);
            }
        }

        // Score promotions
        if mov & (1 << 12) != 0 { // Promotion flag
            score += PIECE_VALUES[4]; // Queen value
        }

        // Score castling moves
        if mov & (1 << 13) != 0 { // Castle flag
            score += 50;
        }

        score
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capture_ordering() {
        let game = Game::new();
        let position = Position::read_FEN(
            "r1bqkbnr/pppp1ppp/8/4p3/2B1P3/8/PPPP1PPP/RNBQK1NR b KQkq - 0 1",
            &game
        );
        
        println!("Position:\n{}", position.to_string());
        println!("Active color: {:?}", position.active_color);
        
        // Convert bitboards to moves
        let mut moves = Vec::new();
        for (i, legal_moves_bitboard) in position.piece_legal_moves.iter().enumerate() {
            if *legal_moves_bitboard == 0 {
                continue;
            }
            let from_square = bit_scan(position.pieces[i].position) as u64;
            for to_square in extract_bits(*legal_moves_bitboard) {
                // Encode move: from_square in lower 6 bits, to_square in next 6 bits
                let mov = from_square | ((to_square as u64) << 6);
                moves.push(mov);
            }
        }
        
        println!("Number of legal moves: {}", moves.len());
        
        // Print each move's details
        for mov in &moves {
            let from_sq = mov & 0x3F;
            let to_sq = (mov >> 6) & 0x3F;
            println!("Move: from square {} to square {}", from_sq, to_sq);
        }
        
        let mut orderer = MoveOrderer::new();
        let ordered_moves = orderer.order_moves(&position, &moves, &game);
        println!("Number of ordered moves: {}", ordered_moves.len());

        if !ordered_moves.is_empty() {
            let first_move = ordered_moves[0];
            let from_sq = first_move & 0x3F;
            let to_sq = (first_move >> 6) & 0x3F;
            println!("First move: from square {} to square {}", from_sq, to_sq);
            assert!(position.is_capture(first_move));
        } else {
            panic!("No moves were generated!");
        }
    }

    #[test]
    fn test_promotion_ordering() {
        let game = Game::new();
        // Position with pawn promotion possible
        let position = Position::read_FEN(
            "8/4P3/8/8/8/8/8/k1K5 w - - 0 1",
            &game
        );
        
        // Convert bitboards to moves
        let mut moves = Vec::new();
        for (i, legal_moves_bitboard) in position.piece_legal_moves.iter().enumerate() {
            if *legal_moves_bitboard == 0 {
                continue;
            }
            let piece = &position.pieces[i];
            let from_square = bit_scan(piece.position) as u64;
            for to_square in extract_bits(*legal_moves_bitboard) {
                // Encode move: from_square in lower 6 bits, to_square in next 6 bits
                let mut mov = from_square | ((to_square as u64) << 6);
                
                // Set promotion flag for pawns moving to the last rank
                if piece.piece_type == PieceType::Pawn {
                    let to_rank = to_square / 8;
                    if (piece.color == Color::White && to_rank == 7) || 
                       (piece.color == Color::Black && to_rank == 0) {
                        mov |= 1 << 12;  // Set promotion flag
                    }
                }
                moves.push(mov);
            }
        }
        
        let mut orderer = MoveOrderer::new();
        let ordered_moves = orderer.order_moves(&position, &moves, &game);

        // Verify that promotions are ordered first
        if !ordered_moves.is_empty() {
            let first_move = ordered_moves[0];
            assert!(position.is_promotion(first_move));
        }
    }
} 