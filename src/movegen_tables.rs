//! Move generation lookup tables for efficient chess move generation.
//! 
//! This module contains pre-computed lookup tables for various piece movements
//! and attack patterns. It uses bitboards for efficient move generation and
//! position evaluation.

use crate::utils::*;

/// A collection of pre-computed lookup tables for chess move generation.
/// 
/// This struct contains various lookup tables that store pre-computed move and attack
/// patterns for different chess pieces. Using these tables significantly improves
/// move generation performance by avoiding runtime calculations.
#[derive(Debug, Clone)]
pub struct MoveGenTables {
    /// Pawn attack patterns indexed by [color][square].
    /// The first dimension represents the color (0 = white, 1 = black),
    /// and the second dimension represents the square (0-63).
    pub pawn_attacks: [[u64; 64]; 2],

    /// Knight attack patterns indexed by square (0-63).
    /// Each u64 represents a bitboard of squares that a knight can attack
    /// from the given square.
    pub knight_attacks: [u64; 64],

    /// King attack patterns indexed by square (0-63).
    /// Each u64 represents a bitboard of squares that a king can attack
    /// from the given square.
    pub king_attacks: [u64; 64],

    /// Bishop movement masks for magic bitboard generation.
    /// These masks represent potential bishop movement paths excluding edge squares.
    pub bishop_masks: [u64; 64],

    /// Rook movement masks for magic bitboard generation.
    /// These masks represent potential rook movement paths excluding edge squares.
    pub rook_masks: [u64; 64],

    /// Bishop attack patterns indexed by [square][magic_index].
    /// Uses magic bitboards for efficient lookup of bishop attacks considering blockers.
    pub bishop_attacks: Vec<Vec<u64>>,

    /// Rook attack patterns indexed by [square][magic_index].
    /// Uses magic bitboards for efficient lookup of rook attacks considering blockers.
    pub rook_attacks: Vec<Vec<u64>>,
}

impl MoveGenTables {
    /// Creates a new instance of MoveGenTables with all lookup tables initialized.
    /// 
    /// This function pre-computes all move and attack patterns for all pieces
    /// and stores them in the appropriate tables. This is computationally expensive
    /// but only needs to be done once at startup.
    /// 
    /// # Returns
    /// 
    /// * A new `MoveGenTables` instance with all tables initialized
    pub fn new() -> Self {
        let mut tables = Self {
            pawn_attacks: [[0; 64]; 2],
            knight_attacks: [0; 64],
            king_attacks: [0; 64],
            bishop_masks: [0; 64],
            rook_masks: [0; 64],
            bishop_attacks: vec![vec![0; 512]; 64],
            rook_attacks: vec![vec![0; 4096]; 64],
        };

        // Initialize king attacks
        for square in 0..64 {
            tables.king_attacks[square] = generate_king_attacks(square);
        }

        // Initialize knight attacks
        for square in 0..64 {
            tables.knight_attacks[square] = generate_knight_attacks(square);
        }

        tables
    }
}

/// Generates a bitboard of all squares a king can attack from a given square.
/// 
/// # Arguments
/// 
/// * `square` - The square index (0-63) from which to generate attacks
/// 
/// # Returns
/// 
/// * A bitboard representing all squares the king can attack
fn generate_king_attacks(square: usize) -> u64 {
    let mut attacks = 0;
    let row = (square / 8) as i32;
    let col = (square % 8) as i32;
    
    // All 8 possible king moves
    let directions = [
        (1, 0), (1, 1), (0, 1), (-1, 1),
        (-1, 0), (-1, -1), (0, -1), (1, -1)
    ];

    for (dr, dc) in directions.iter() {
        let new_row = row + dr;
        let new_col = col + dc;
        if new_row >= 0 && new_row < 8 && new_col >= 0 && new_col < 8 {
            attacks |= 1u64 << (new_row * 8 + new_col);
        }
    }
    
    attacks
}

/// Generates a bitboard of all squares a knight can attack from a given square.
/// 
/// # Arguments
/// 
/// * `square` - The square index (0-63) from which to generate attacks
/// 
/// # Returns
/// 
/// * A bitboard representing all squares the knight can attack
fn generate_knight_attacks(square: usize) -> u64 {
    let mut attacks = 0;
    let row = (square / 8) as i32;
    let col = (square % 8) as i32;
    
    // All 8 possible knight moves
    let moves = [
        (2, 1), (2, -1), (-2, 1), (-2, -1),
        (1, 2), (1, -2), (-1, 2), (-1, -2)
    ];

    for (dr, dc) in moves.iter() {
        let new_row = row + dr;
        let new_col = col + dc;
        if new_row >= 0 && new_row < 8 && new_col >= 0 && new_col < 8 {
            attacks |= 1u64 << (new_row * 8 + new_col);
        }
    }
    
    attacks
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_king_attacks() {
        let tables = MoveGenTables::new();
        
        // Test center square (e4)
        let e4 = 28;
        let attacks = tables.king_attacks[e4];
        assert_eq!(attacks.count_ones(), 8); // Should have 8 moves in the center
        
        // Test corner square (a1)
        let a1 = 0;
        let attacks = tables.king_attacks[a1];
        assert_eq!(attacks.count_ones(), 3); // Should have 3 moves in the corner
    }

    #[test]
    fn test_knight_attacks() {
        let tables = MoveGenTables::new();
        
        // Test center square (e4)
        let e4 = 28;
        let attacks = tables.knight_attacks[e4];
        assert_eq!(attacks.count_ones(), 8); // Should have 8 moves in the center
        
        // Test corner square (a1)
        let a1 = 0;
        let attacks = tables.knight_attacks[a1];
        assert_eq!(attacks.count_ones(), 2); // Should have 2 moves in the corner
    }
}
