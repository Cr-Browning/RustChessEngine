use crate::position::{Position, Color, PieceType};
use crate::utils::bit_scan;

// Material values in centipawns (1 pawn = 100)
const PAWN_VALUE: i32 = 100;
const KNIGHT_VALUE: i32 = 320;
const BISHOP_VALUE: i32 = 330;
const ROOK_VALUE: i32 = 500;
const QUEEN_VALUE: i32 = 900;

// Piece-square tables for positional bonuses
// Values are in centipawns and are from White's perspective
// For Black, we'll flip the table vertically
const PAWN_TABLE: [i32; 64] = [
    0,  0,  0,  0,  0,  0,  0,  0,
    50, 50, 50, 50, 50, 50, 50, 50,
    10, 10, 20, 30, 30, 20, 10, 10,
    5,  5, 10, 25, 25, 10,  5,  5,
    0,  0,  0, 20, 20,  0,  0,  0,
    5, -5,-10,  0,  0,-10, -5,  5,
    5, 10, 10,-20,-20, 10, 10,  5,
    0,  0,  0,  0,  0,  0,  0,  0
];

const KNIGHT_TABLE: [i32; 64] = [
    -50,-40,-30,-30,-30,-30,-40,-50,
    -40,-20,  0,  0,  0,  0,-20,-40,
    -30,  0, 10, 15, 15, 10,  0,-30,
    -30,  5, 15, 20, 20, 15,  5,-30,
    -30,  0, 15, 20, 20, 15,  0,-30,
    -30,  5, 10, 15, 15, 10,  5,-30,
    -40,-20,  0,  5,  5,  0,-20,-40,
    -50,-40,-30,-30,-30,-30,-40,-50
];

const BISHOP_TABLE: [i32; 64] = [
    -20,-10,-10,-10,-10,-10,-10,-20,
    -10,  0,  0,  0,  0,  0,  0,-10,
    -10,  0,  5, 10, 10,  5,  0,-10,
    -10,  5,  5, 10, 10,  5,  5,-10,
    -10,  0, 10, 10, 10, 10,  0,-10,
    -10, 10, 10, 10, 10, 10, 10,-10,
    -10,  5,  0,  0,  0,  0,  5,-10,
    -20,-10,-10,-10,-10,-10,-10,-20
];

const ROOK_TABLE: [i32; 64] = [
    0,  0,  0,  0,  0,  0,  0,  0,
    5, 10, 10, 10, 10, 10, 10,  5,
    -5,  0,  0,  0,  0,  0,  0, -5,
    -5,  0,  0,  0,  0,  0,  0, -5,
    -5,  0,  0,  0,  0,  0,  0, -5,
    -5,  0,  0,  0,  0,  0,  0, -5,
    -5,  0,  0,  0,  0,  0,  0, -5,
    0,  0,  0,  5,  5,  0,  0,  0
];

const QUEEN_TABLE: [i32; 64] = [
    -20,-10,-10, -5, -5,-10,-10,-20,
    -10,  0,  0,  0,  0,  0,  0,-10,
    -10,  0,  5,  5,  5,  5,  0,-10,
    -5,  0,  5,  5,  5,  5,  0, -5,
    0,  0,  5,  5,  5,  5,  0, -5,
    -10,  5,  5,  5,  5,  5,  0,-10,
    -10,  0,  5,  0,  0,  0,  0,-10,
    -20,-10,-10, -5, -5,-10,-10,-20
];

const KING_MIDDLEGAME_TABLE: [i32; 64] = [
    -30,-40,-40,-50,-50,-40,-40,-30,
    -30,-40,-40,-50,-50,-40,-40,-30,
    -30,-40,-40,-50,-50,-40,-40,-30,
    -30,-40,-40,-50,-50,-40,-40,-30,
    -20,-30,-30,-40,-40,-30,-30,-20,
    -10,-20,-20,-20,-20,-20,-20,-10,
    20, 20,  0,  0,  0,  0, 20, 20,
    20, 30, 10,  0,  0, 10, 30, 20
];

// Pawn structure bonuses/penalties
const CENTRAL_PAWN_BONUS: i32 = 20;  // Bonus for controlling central squares (e4,d4,e5,d5)
const DOUBLED_PAWN_PENALTY: i32 = -20;  // Penalty for doubled pawns
const ISOLATED_PAWN_PENALTY: i32 = -10;  // Penalty for isolated pawns

// Central squares for pawn evaluation
const CENTRAL_SQUARES: u64 = 0x0000001818000000;  // e4,d4,e5,d5

// Additional positional bonuses
const SPACE_BONUS: i32 = 10;  // Bonus for each pawn advanced beyond rank 3/4
const CENTER_CONTROL_BONUS: i32 = 15;  // Bonus for controlling e4/d4 vs e5/d5
const DEVELOPMENT_BONUS: i32 = 10;  // Bonus for each piece that can develop

// Center squares (e4,d4 for White, e5,d5 for Black)
const WHITE_CENTER: u64 = 0x0000001818000000;  // e4,d4
const BLACK_CENTER: u64 = 0x0000000000181800;  // e5,d5

pub struct Evaluation {
    position: Position,
}

impl Evaluation {
    pub fn new(position: Position) -> Self {
        Evaluation { position }
    }

    /// Evaluates a chess position from White's perspective.
    /// Returns a score in centipawns, positive for White advantage, negative for Black advantage.
    pub fn evaluate_position(&self) -> i32 {
        let material_score = self.evaluate_material();
        let positional_score = self.evaluate_piece_positions();
        
        // Always return score from White's perspective
        material_score + positional_score
    }

    /// Evaluates material balance of the position
    fn evaluate_material(&self) -> i32 {
        let mut score = 0;
        
        for piece in &self.position.pieces {
            // Skip captured pieces
            if piece.position == 0 {
                continue;
            }

            let piece_value = match piece.piece_type {
                PieceType::Pawn => PAWN_VALUE,
                PieceType::Knight => KNIGHT_VALUE,
                PieceType::Bishop => BISHOP_VALUE,
                PieceType::Rook => ROOK_VALUE,
                PieceType::Queen => QUEEN_VALUE,
                PieceType::King => 0, // King has no material value
            };
            
            if piece.color == Color::White {
                score += piece_value;
            } else {
                score -= piece_value;
            }
        }
        
        score
    }

    /// Evaluates piece positions using piece-square tables
    fn evaluate_piece_positions(&self) -> i32 {
        let mut score = 0;
        
        // Get all pawns for each color
        let mut white_pawns = 0u64;
        let mut black_pawns = 0u64;
        
        for piece in &self.position.pieces {
            // Skip captured pieces
            if piece.position == 0 {
                continue;
            }

            if piece.piece_type == PieceType::Pawn {
                if piece.color == Color::White {
                    white_pawns |= piece.position;
                } else {
                    black_pawns |= piece.position;
                }
            }

            // Basic piece square table evaluation
            let square = bit_scan(piece.position);
            let table_index = if piece.color == Color::White {
                square
            } else {
                63 - square // Flip for black pieces
            };
            
            let position_value = match piece.piece_type {
                PieceType::Pawn => PAWN_TABLE[table_index],
                PieceType::Knight => KNIGHT_TABLE[table_index],
                PieceType::Bishop => BISHOP_TABLE[table_index],
                PieceType::Rook => ROOK_TABLE[table_index],
                PieceType::Queen => QUEEN_TABLE[table_index],
                PieceType::King => KING_MIDDLEGAME_TABLE[table_index],
            };
            
            if piece.color == Color::White {
                score += position_value;
            } else {
                score -= position_value;
            }
        }

        // Evaluate pawn structure
        score += self.evaluate_pawn_structure(white_pawns, black_pawns);
        
        // Evaluate space and center control
        score += self.evaluate_space_and_center(white_pawns, black_pawns);
        
        score
    }

    fn evaluate_pawn_structure(&self, white_pawns: u64, black_pawns: u64) -> i32 {
        let mut score = 0;

        // Central pawn control
        score += (white_pawns & CENTRAL_SQUARES).count_ones() as i32 * CENTRAL_PAWN_BONUS;
        score -= (black_pawns & CENTRAL_SQUARES).count_ones() as i32 * CENTRAL_PAWN_BONUS;

        // Evaluate doubled pawns (multiple pawns on same file)
        for file in 0..8 {
            let file_mask = 0x0101010101010101u64 << file;
            let white_pawns_in_file = (white_pawns & file_mask).count_ones();
            let black_pawns_in_file = (black_pawns & file_mask).count_ones();
            
            if white_pawns_in_file > 1 {
                score += DOUBLED_PAWN_PENALTY * (white_pawns_in_file - 1) as i32;
            }
            if black_pawns_in_file > 1 {
                score -= DOUBLED_PAWN_PENALTY * (black_pawns_in_file - 1) as i32;
            }
        }

        // Evaluate isolated pawns (no friendly pawns on adjacent files)
        for file in 0..8 {
            let file_mask = 0x0101010101010101u64 << file;
            let adjacent_files_mask = if file == 0 {
                0x0202020202020202u64 // Only right file
            } else if file == 7 {
                0x4040404040404040u64 // Only left file
            } else {
                (0x0101010101010101u64 << (file - 1)) | (0x0101010101010101u64 << (file + 1))
            };

            // Check white pawns
            if (white_pawns & file_mask) != 0 && (white_pawns & adjacent_files_mask) == 0 {
                score += ISOLATED_PAWN_PENALTY;
            }
            // Check black pawns
            if (black_pawns & file_mask) != 0 && (black_pawns & adjacent_files_mask) == 0 {
                score -= ISOLATED_PAWN_PENALTY;
            }
        }

        score
    }

    fn evaluate_space_and_center(&self, white_pawns: u64, black_pawns: u64) -> i32 {
        let mut score = 0;

        // Space advantage - count pawns beyond rank 3 for White, rank 6 for Black
        let white_advanced = white_pawns & 0x00FFFFFF000000;  // Ranks 4-6
        let black_advanced = black_pawns & 0x000000FFFFFF00;  // Ranks 3-5
        score += (white_advanced.count_ones() as i32) * SPACE_BONUS;
        score -= (black_advanced.count_ones() as i32) * SPACE_BONUS;

        // Center control
        let white_center_control = white_pawns & WHITE_CENTER;
        let black_center_control = black_pawns & BLACK_CENTER;
        score += (white_center_control.count_ones() as i32) * CENTER_CONTROL_BONUS;
        score -= (black_center_control.count_ones() as i32) * CENTER_CONTROL_BONUS;

        // Development potential - check if center pawns have moved
        if (white_pawns & 0x0000000000001000) == 0 {  // e2 pawn moved
            score += DEVELOPMENT_BONUS;  // Light squared bishop can develop
        }
        if (black_pawns & 0x0010000000000000) == 0 {  // e7 pawn moved
            score -= DEVELOPMENT_BONUS;  // Light squared bishop can develop
        }

        score
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Game;

    #[test]
    fn test_initial_position_evaluation() {
        let game = Game::new();
        let position = Position::new(&game);
        let evaluation = Evaluation::new(position);
        
        // Initial position should be equal (score close to 0)
        assert_eq!(evaluation.evaluate_position(), 0);
    }

    #[test]
    fn test_material_advantage_white() {
        let game = Game::new();
        // Position where White is up a knight
        let position = Position::read_FEN(
            "rnbqkb1r/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
            &game
        );
        let evaluation = Evaluation::new(position);
        
        // White should be up roughly a knight's value
        assert!(evaluation.evaluate_position() >= KNIGHT_VALUE - 50);
        assert!(evaluation.evaluate_position() <= KNIGHT_VALUE + 50);
    }

    #[test]
    fn test_pawn_structure_evaluation() {
        let game = Game::new();
        // Position with better pawn structure for White
        let position = Position::read_FEN(
            "rnbqkbnr/ppp1pppp/8/3p4/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 1",
            &game
        );
        let evaluation = Evaluation::new(position);
        
        let material_score = evaluation.evaluate_material();
        let positional_score = evaluation.evaluate_piece_positions();
        let total_score = evaluation.evaluate_position();
        
        println!("Material score: {}", material_score);
        println!("Positional score: {}", positional_score);
        println!("Total score: {}", total_score);
        
        // White's better pawn structure should give a positive score
        assert!(evaluation.evaluate_position() > 0);
    }
}
