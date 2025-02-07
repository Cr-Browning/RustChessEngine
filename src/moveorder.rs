use crate::position::Position;
use crate::Game;
use crate::utils::*;
use crate::position::*;
use crate::chess_move::*;


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
        let from_square = mov & 0x3F;  // Extract from_square from bits 0-5
        let to_square = (mov >> 6) & 0x3F;  // Extract to_square from bits 6-11

        // Get the moving piece
        if let Some(piece_idx) = position.squares[from_square as usize].get_piece_index() {
            let moving_piece = &position.pieces[piece_idx];
            
            // Score captures
            if let Some(target_idx) = position.squares[to_square as usize].get_piece_index() {
                let target_piece = &position.pieces[target_idx];
                if target_piece.position != 0 && target_piece.color != moving_piece.color {
                    // MVV-LVA scoring: Most Valuable Victim - Least Valuable Attacker
                    let victim_value = PIECE_VALUES[target_piece.piece_type as usize];
                    let attacker_value = PIECE_VALUES[moving_piece.piece_type as usize];
                    score += CAPTURE_SCORE_BASE + victim_value - (attacker_value / 100);
                }
            }
            
            // Score promotions
            if (mov & (1 << 12)) != 0 {
                score += 100000;  // Much higher than any capture
            }
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
        
        // Print each piece's position and legal moves
        for (i, piece) in position.pieces.iter().enumerate() {
            if piece.position == 0 {
                continue;
            }
            println!("Piece {}: {:?} {:?} at square {}, legal moves: {:?}", 
                i, piece.color, piece.piece_type, 
                bit_scan_safe(piece.position).unwrap_or(64),
                extract_bits(position.piece_legal_moves[i]));
        }
        
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
            println!("Move: from square {} to square {}, is_capture: {}", 
                from_sq, to_sq, position.is_capture(*mov));
        }
        
        let mut orderer = MoveOrderer::new();
        let ordered_moves = orderer.order_moves(&position, &moves, &game);
        println!("Number of ordered moves: {}", ordered_moves.len());

        // Print scores for each move
        for mov in &ordered_moves {
            let from_sq = mov & 0x3F;
            let to_sq = (mov >> 6) & 0x3F;
            let score = orderer.score_move(&position, *mov, &game);
            println!("Move from {} to {}, score: {}, is_capture: {}", 
                from_sq, to_sq, score, position.is_capture(*mov));
        }

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
        
        println!("Position:\n{}", position.to_string());
        
        // Convert bitboards to moves
        let mut moves = Vec::new();
        for (i, legal_moves_bitboard) in position.piece_legal_moves.iter().enumerate() {
            if *legal_moves_bitboard == 0 {
                continue;
            }
            let piece = &position.pieces[i];
            println!("Piece at index {}: {:?} {:?} at square {}", i, piece.color, piece.piece_type, bit_scan(piece.position));
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
                println!("  Move from {} to {}, promotion: {}", from_square, to_square, mov & (1 << 12) != 0);
                moves.push(mov);
            }
        }
        
        let mut orderer = MoveOrderer::new();
        let ordered_moves = orderer.order_moves(&position, &moves, &game);

        // Verify that promotions are ordered first
        if !ordered_moves.is_empty() {
            let first_move = ordered_moves[0];
            let from_square = first_move & 0x3F;
            let to_square = (first_move >> 6) & 0x3F;
            println!("First move: from {} to {}, promotion: {}", from_square, to_square, first_move & (1 << 12) != 0);
            assert!(position.is_promotion(first_move));
        }
    }
} 