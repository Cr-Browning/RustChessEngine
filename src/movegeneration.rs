//! Chess move generation module.
//! 
//! This module handles the generation of legal chess moves for all piece types.
//! It uses bitboard operations for efficient move generation and validates moves
//! against the current game state.

use crate::position::*;
#[allow(unused_imports)]
use crate::knightattacks::*;
use crate::position::PieceType::*;
use crate::utils::{bit_scan_safe, extract_bits};
use crate::Game;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CastlingSide {
    Kingside,
    Queenside,
}

/// Generates all legal moves for the current position.
/// 
/// This function iterates through all pieces of the active color and generates
/// their legal moves considering the current board state, including captures,
/// en passant, and castling rights.
/// 
/// # Arguments
/// 
/// * `game` - Reference to the current game state
/// 
/// # Returns
/// 
/// * A vector of new positions, each representing a legal move
pub fn generate_moves(game: &Game) -> Vec<Position> {
    let mut new_positions = Vec::with_capacity(32);
    let position = &game.position;
    
    let own_occupancy = if position.active_color == Color::White {
        position.white_occupancy
    } else {
        position.black_occupancy
    };

    let opponent_occupancy = if position.active_color == Color::White {
        position.black_occupancy
    } else {
        position.white_occupancy
    };

    let all_occupancy = own_occupancy | opponent_occupancy;

    for piece in position.pieces.iter().filter(|p| p.color == position.active_color) {
        match piece.piece_type {
            Pawn => {
                let moves = generate_pawn_moves(piece, game, all_occupancy, opponent_occupancy);
                new_positions.extend(moves);
            }
            Knight => {
                let moves = generate_knight_moves(piece, game, own_occupancy);
                new_positions.extend(moves);
            }
            Bishop => {
                let moves = generate_bishop_moves(piece, game, own_occupancy, all_occupancy);
                new_positions.extend(moves);
            }
            Rook => {
                let moves = generate_rook_moves(piece, game, own_occupancy, all_occupancy);
                new_positions.extend(moves);
            }
            Queen => {
                let moves = generate_queen_moves(piece, game, own_occupancy, all_occupancy);
                new_positions.extend(moves);
            }
            King => {
                let moves = generate_king_moves(piece, game, own_occupancy, all_occupancy);
                new_positions.extend(moves);
            }
        }
    }

    // Check castling for kings
    if let Some(king) = position.pieces.iter().find(|p| p.piece_type == King && p.color == position.active_color) {
        if can_castle(position, position.active_color, CastlingSide::Kingside) {
            add_castling_moves(king, game, &mut new_positions, CastlingSide::Kingside);
        }
        if can_castle(position, position.active_color, CastlingSide::Queenside) {
            add_castling_moves(king, game, &mut new_positions, CastlingSide::Queenside);
        }
    }

    new_positions
}

/// Generates all legal pawn moves for a given piece.
/// 
/// This includes:
/// - Single square advances
/// - Double square advances from starting position
/// - Diagonal captures
/// - En passant captures
/// 
/// # Arguments
/// 
/// * `piece` - The pawn piece to generate moves for
/// * `game` - Reference to the current game state
/// * `all_occupancy` - Bitboard of all pieces on the board
/// * `opponent_occupancy` - Bitboard of opponent pieces
/// 
/// # Returns
/// 
/// * A vector of new positions representing legal pawn moves
fn generate_pawn_moves(piece: &Piece, game: &Game, all_occupancy: u64, opponent_occupancy: u64) -> Vec<Position> {
    let mut new_positions = Vec::new();
    if piece.position == 0 {
        return new_positions;  // Skip captured pieces
    }
    if let Some(square) = bit_scan_safe(piece.position) {
        // Use the correct forward and diagonal moves based on color
        let (forward_moves, diagonal_moves) = match piece.color {
            Color::White => (
                game.pawn_attacks.white_forward_moves[square],
                game.pawn_attacks.white_diagonal_moves[square]
            ),
            Color::Black => (
                game.pawn_attacks.black_forward_moves[square],
                game.pawn_attacks.black_diagonal_moves[square]
            ),
        };
        
        // Forward moves (not blocked)
        let single_forward = forward_moves & !all_occupancy;
        let double_forward = if piece.color == Color::White && square / 8 == 1 {
            // For white pawns on second rank, check if both squares are empty
            let single_empty = (forward_moves & !all_occupancy) != 0;
            if single_empty {
                forward_moves & !all_occupancy & (0xFF << 16) // Only allow double moves to rank 4
            } else {
                0
            }
        } else if piece.color == Color::Black && square / 8 == 6 {
            // For black pawns on seventh rank, check if both squares are empty
            let single_empty = (forward_moves & !all_occupancy) != 0;
            if single_empty {
                forward_moves & !all_occupancy & (0xFF << 32) // Only allow double moves to rank 5
            } else {
                0
            }
        } else {
            0
        };
        
        // Add single moves
        for target in extract_bits(single_forward) {
            let mut new_position = game.position.clone();
            new_position.move_piece(piece.position, target, game);
            new_positions.push(new_position);
        }
        
        // Add double moves
        for target in extract_bits(double_forward) {
            let mut new_position = game.position.clone();
            new_position.move_piece(piece.position, target, game);
            new_positions.push(new_position);
        }
        
        // Diagonal captures
        let captures = diagonal_moves & opponent_occupancy;
        for target in extract_bits(captures) {
            let mut new_position = game.position.clone();
            new_position.move_piece(piece.position, target, game);
            new_positions.push(new_position);
        }
        
        // En passant
        if let Some(en_passant) = game.position.en_passant {
            let en_passant_captures = diagonal_moves & en_passant;
            if en_passant_captures != 0 {
                if let Some(target) = bit_scan_safe(en_passant) {
                    let mut new_position = game.position.clone();
                    new_position.move_piece(piece.position, target, game);
                    new_positions.push(new_position);
                }
            }
        }
    }
    
    new_positions
}

/// Generates all legal knight moves for a given piece.
/// 
/// # Arguments
/// 
/// * `piece` - The knight piece to generate moves for
/// * `game` - Reference to the current game state
/// * `own_occupancy` - Bitboard of friendly pieces
/// 
/// # Returns
/// 
/// * A vector of new positions representing legal knight moves
fn generate_knight_moves(piece: &Piece, game: &Game, own_occupancy: u64) -> Vec<Position> {
    let mut new_positions = Vec::new();
    if piece.position == 0 {
        return new_positions;  // Skip captured pieces
    }
    if let Some(square) = bit_scan_safe(piece.position) {
        let mut attacks = game.move_gen_tables.knight_attacks[square];
        attacks &= !own_occupancy;
        let potential_moves = extract_bits(attacks);
        for pmove in potential_moves {
            let mut new_position = game.position.clone();
            new_position.move_piece(piece.position, pmove, game);
            new_positions.push(new_position);
        }
    }
    new_positions
}

/// Generates all legal bishop moves for a given piece.
/// 
/// # Arguments
/// 
/// * `piece` - The bishop piece to generate moves for
/// * `game` - Reference to the current game state
/// * `own_occupancy` - Bitboard of friendly pieces
/// * `all_occupancy` - Bitboard of all pieces on the board
/// 
/// # Returns
/// 
/// * A vector of new positions representing legal bishop moves
fn generate_bishop_moves(piece: &Piece, game: &Game, own_occupancy: u64, all_occupancy: u64) -> Vec<Position> {
    let mut new_positions = Vec::new();
    if piece.position == 0 {
        return new_positions;  // Skip captured pieces
    }
    if let Some(square) = bit_scan_safe(piece.position) {
        let attacks = game.rays.get_bishop_attacks(square, all_occupancy, piece.color, own_occupancy);
        let valid_moves = attacks & !own_occupancy;
        
        for target in extract_bits(valid_moves) {
            let mut new_position = game.position.clone();
            new_position.move_piece(piece.position, target, game);
            new_positions.push(new_position);
        }
    }
    new_positions
}

/// Generates all legal rook moves for a given piece.
/// 
/// # Arguments
/// 
/// * `piece` - The rook piece to generate moves for
/// * `game` - Reference to the current game state
/// * `own_occupancy` - Bitboard of friendly pieces
/// * `all_occupancy` - Bitboard of all pieces on the board
/// 
/// # Returns
/// 
/// * A vector of new positions representing legal rook moves
fn generate_rook_moves(piece: &Piece, game: &Game, own_occupancy: u64, all_occupancy: u64) -> Vec<Position> {
    let mut new_positions = Vec::new();
    if piece.position == 0 {
        return new_positions;  // Skip captured pieces
    }
    if let Some(square) = bit_scan_safe(piece.position) {
        let attacks = game.rays.get_rook_attacks(square, all_occupancy);
        let valid_moves = attacks & !own_occupancy;
        
        for target in extract_bits(valid_moves) {
            let mut new_position = game.position.clone();
            new_position.move_piece(piece.position, target, game);
            new_positions.push(new_position);
        }
    }
    new_positions
}

/// Generates all legal queen moves for a given piece.
/// 
/// Combines bishop and rook move generation since a queen
/// moves like both pieces combined.
/// 
/// # Arguments
/// 
/// * `piece` - The queen piece to generate moves for
/// * `game` - Reference to the current game state
/// * `own_occupancy` - Bitboard of friendly pieces
/// * `all_occupancy` - Bitboard of all pieces on the board
/// 
/// # Returns
/// 
/// * A vector of new positions representing legal queen moves
fn generate_queen_moves(piece: &Piece, game: &Game, own_occupancy: u64, all_occupancy: u64) -> Vec<Position> {
    let mut new_positions = Vec::new();
    if piece.position == 0 {
        return new_positions;  // Skip captured pieces
    }
    if let Some(square) = bit_scan_safe(piece.position) {
        let attacks = game.rays.get_queen_attacks(square, all_occupancy);
        let valid_moves = attacks & !own_occupancy;
        
        for target in extract_bits(valid_moves) {
            let mut new_position = game.position.clone();
            new_position.move_piece(piece.position, target, game);
            new_positions.push(new_position);
        }
    }
    new_positions
}

/// Generates all legal king moves for a given piece.
/// 
/// Includes both regular moves and castling moves if available.
/// 
/// # Arguments
/// 
/// * `piece` - The king piece to generate moves for
/// * `game` - Reference to the current game state
/// * `own_occupancy` - Bitboard of friendly pieces
/// * `all_occupancy` - Bitboard of all pieces on the board
/// 
/// # Returns
/// 
/// * A vector of new positions representing legal king moves
fn generate_king_moves(piece: &Piece, game: &Game, own_occupancy: u64, all_occupancy: u64) -> Vec<Position> {
    let mut new_positions = Vec::new();
    if piece.position == 0 {
        return new_positions;  // Skip captured pieces
    }
    if let Some(square) = bit_scan_safe(piece.position) {
        let mut attacks = game.move_gen_tables.king_attacks[square];
        attacks &= !own_occupancy;
        
        // Normal moves
        for target in extract_bits(attacks) {
            let mut new_position = game.position.clone();
            new_position.move_piece(piece.position, target, game);
            new_positions.push(new_position);
        }
        
        // Castling moves
        if can_castle(&game.position, piece.color, CastlingSide::Kingside) {
            add_castling_moves(piece, game, &mut new_positions, CastlingSide::Kingside);
        }
        if can_castle(&game.position, piece.color, CastlingSide::Queenside) {
            add_castling_moves(piece, game, &mut new_positions, CastlingSide::Queenside);
        }
    }
    new_positions
}

/// Checks if castling is legal in the current position.
/// 
/// # Arguments
/// 
/// * `position` - Reference to the current game state
/// * `color` - The color of the king
/// * `side` - The castling side
/// 
/// # Returns
/// 
/// * `true` if castling is legal, `false` otherwise
pub fn can_castle(position: &Position, color: Color, side: CastlingSide) -> bool {
    // Check if the king has moved
    if (color == Color::White && position.white_king_moved) ||
       (color == Color::Black && position.black_king_moved) {
        return false;
    }

    // Check if the appropriate rook has moved
    match (color, side) {
        (Color::White, CastlingSide::Kingside) => {
            if position.white_kingside_rook_moved {
                return false;
            }
        },
        (Color::White, CastlingSide::Queenside) => {
            if position.white_queenside_rook_moved {
                return false;
            }
        },
        (Color::Black, CastlingSide::Kingside) => {
            if position.black_kingside_rook_moved {
                return false;
            }
        },
        (Color::Black, CastlingSide::Queenside) => {
            if position.black_queenside_rook_moved {
                return false;
            }
        },
    }

    // Check if the path is blocked by any pieces
    let path = match (color, side) {
        (Color::White, CastlingSide::Kingside) => 0x60,  // f1 and g1
        (Color::White, CastlingSide::Queenside) => 0xE,  // b1, c1, and d1
        (Color::Black, CastlingSide::Kingside) => 0x6000000000000000,  // f8 and g8
        (Color::Black, CastlingSide::Queenside) => 0xE00000000000000,  // b8, c8, and d8
    };

    let all_pieces = position.white_occupancy | position.black_occupancy;
    if (path & all_pieces) != 0 {
        return false;
    }

    // Check if the castling path is attacked for the correct side
    match (color, side) {
        (Color::White, CastlingSide::Kingside) => {
            if position.white_kingside_path_attacked {
                return false;
            }
        },
        (Color::White, CastlingSide::Queenside) => {
            if position.white_queenside_path_attacked {
                return false;
            }
        },
        (Color::Black, CastlingSide::Kingside) => {
            if position.black_kingside_path_attacked {
                return false;
            }
        },
        (Color::Black, CastlingSide::Queenside) => {
            if position.black_queenside_path_attacked {
                return false;
            }
        },
    }

    // Check castling rights
    let required_rights = match (color, side) {
        (Color::White, CastlingSide::Kingside) => CastlingRights::WHITEKINGSIDE,
        (Color::White, CastlingSide::Queenside) => CastlingRights::WHITEQUEENSIDE,
        (Color::Black, CastlingSide::Kingside) => CastlingRights::BLACKKINGSIDE,
        (Color::Black, CastlingSide::Queenside) => CastlingRights::BLACKQUEENSIDE,
    };

    if position.castling_rights & required_rights == CastlingRights::NONE {
        return false;
    }

    true
}

/// Adds legal castling moves to the list of moves.
/// 
/// # Arguments
/// 
/// * `piece` - The king piece to generate castling moves for
/// * `game` - Reference to the current game state
/// * `new_positions` - Vector to add castling moves to
/// * `side` - The castling side
fn add_castling_moves(piece: &Piece, game: &Game, new_positions: &mut Vec<Position>, side: CastlingSide) {
    if piece.position == 0 {
        return;  // Skip captured pieces
    }
    if let Some(king_pos) = bit_scan_safe(piece.position) {
        let mut new_position = game.position.clone();
        let (new_king_pos, new_rook_pos, old_rook_pos) = match (piece.color, side) {
            (Color::White, CastlingSide::Kingside) => (6, 5, 7),   // g1, f1, h1
            (Color::White, CastlingSide::Queenside) => (2, 3, 0),  // c1, d1, a1
            (Color::Black, CastlingSide::Kingside) => (62, 61, 63),  // g8, f8, h8
            (Color::Black, CastlingSide::Queenside) => (58, 59, 56),  // c8, d8, a8
        };

        // Move the king
        let king_piece = new_position.pieces.iter_mut()
            .find(|p| p.piece_type == PieceType::King && p.color == piece.color)
            .unwrap();
        king_piece.position = 1u64 << new_king_pos;

        // Move the rook
        let rook_piece = new_position.pieces.iter_mut()
            .find(|p| p.piece_type == PieceType::Rook && p.color == piece.color && p.position == 1u64 << old_rook_pos)
            .unwrap();
        rook_piece.position = 1u64 << new_rook_pos;

        // Update occupancy bitboards
        if piece.color == Color::White {
            new_position.white_occupancy = new_position.pieces.iter()
                .filter(|p| p.color == Color::White)
                .map(|p| p.position)
                .fold(0, |acc, pos| acc | pos);
        } else {
            new_position.black_occupancy = new_position.pieces.iter()
                .filter(|p| p.color == Color::Black)
                .map(|p| p.position)
                .fold(0, |acc, pos| acc | pos);
        }

        // Update castling flags
        match piece.color {
            Color::White => {
                new_position.white_king_moved = true;
                match side {
                    CastlingSide::Kingside => new_position.white_kingside_rook_moved = true,
                    CastlingSide::Queenside => new_position.white_queenside_rook_moved = true,
                }
            },
            Color::Black => {
                new_position.black_king_moved = true;
                match side {
                    CastlingSide::Kingside => new_position.black_kingside_rook_moved = true,
                    CastlingSide::Queenside => new_position.black_queenside_rook_moved = true,
                }
            },
        }

        new_positions.push(new_position);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::position::*;

    #[test]
    fn test_can_castle_king_moved() {
        let mut game = Game::new();
        game.position.white_king_moved = true;
        assert!(!can_castle(&game.position, Color::White, CastlingSide::Kingside));
    }

    #[test]
    fn test_can_castle_rook_moved() {
        let mut game = Game::new();
        game.position.white_kingside_rook_moved = true;
        assert!(!can_castle(&game.position, Color::White, CastlingSide::Kingside));
    }

    #[test]
    fn test_can_castle_path_blocked() {
        let mut game = Game::new();
        // Place a piece on f1 to block the kingside castling path
        game.position.white_occupancy |= 0x20;
        assert!(!can_castle(&game.position, Color::White, CastlingSide::Kingside));
    }

    #[test]
    fn test_can_castle_path_attacked() {
        let mut game = Game::new();
        game.position.white_kingside_path_attacked = true;
        assert!(!can_castle(&game.position, Color::White, CastlingSide::Kingside));
    }

    #[test]
    fn test_add_castling_moves_kingside() {
        let mut game = Game::new();
        let king = game.position.pieces.iter()
            .find(|p| p.piece_type == PieceType::King && p.color == Color::White)
            .unwrap();
        
        let mut new_positions = Vec::new();
        add_castling_moves(king, &game, &mut new_positions, CastlingSide::Kingside);
        
        assert_eq!(new_positions.len(), 1);
    }

    #[test]
    fn test_add_castling_moves_queenside() {
        let mut game = Game::new();
        let king = game.position.pieces.iter()
            .find(|p| p.piece_type == PieceType::King && p.color == Color::White)
            .unwrap();
        
        let mut new_positions = Vec::new();
        add_castling_moves(king, &game, &mut new_positions, CastlingSide::Queenside);
        
        assert_eq!(new_positions.len(), 1);
    }
}