//! Knight attack pattern generation module.
//! 
//! This module handles the generation of knight attack patterns using bitboards.
//! It pre-computes all possible knight moves from each square for efficient
//! move generation during gameplay.

use crate::utils::*;

/// Type alias for a 64-bit integer representing a chess board
type Bitboard = u64;

/// A structure containing pre-computed knight attack patterns.
/// 
/// This struct wraps a vector of bitboards, where each bitboard represents
/// the squares a knight can attack from a given position on the board.
/// The vector is indexed by the square number (0-63).
#[derive(Debug, Clone)]
pub struct KnightAttacks(pub Vec<Bitboard>);

impl KnightAttacks {
    /// Creates a new instance with pre-computed knight attack patterns.
    /// 
    /// This function initializes attack patterns for all 64 squares on the board.
    /// The patterns are stored in a vector for efficient lookup during move
    /// generation.
    /// 
    /// # Returns
    /// 
    /// * A new `KnightAttacks` instance with all patterns pre-computed
    pub fn new() -> Self {
        let mut attacks = vec![];

        for row in 1..=8 {
            for col in 1..=8 {
                let attacks_from_this_square = knight_attacks(row, col);
                attacks.push(attacks_from_this_square);
            }
        }
        Self(attacks)
    }
}

/// Generates a bitboard of knight attacks from a given square.
/// 
/// This function calculates all valid knight moves from a specific position
/// on the board, considering the L-shaped movement pattern and board boundaries.
/// 
/// # Arguments
/// 
/// * `row` - The row number (1-8) of the knight's position
/// * `col` - The column number (1-8) of the knight's position
/// 
/// # Returns
/// 
/// * A bitboard representing all squares the knight can attack
fn knight_attacks(row: i32, col: i32) -> Bitboard {
    let attack_pairs = [(1,2), (1, -2), (-1, 2),(-1, -2), (2, 1), (2, -1), (-2, 1), (-2, -1)];
    let mut bitboard = 0;

    for (r, c) in attack_pairs.iter() {
        let new_row = row + r;
        let new_col = col + c;
        if new_row >= 1 && new_row <= 8 && new_col >= 1 && new_col <= 8 {
            bitboard |= set_bit(new_row, new_col);
        }
    }
    bitboard
}

#[cfg(test)]
mod tests {
    use super::{knight_attacks, print_bitboard, KnightAttacks};

    /// Tests that KnightAttacks can be initialized without panicking
    #[test]
    fn test_knight_attacks_can_initialize() {
        let knight_attacks = KnightAttacks::new();
    }

    /// Tests knight attack patterns from various squares on the board
    #[test]
    fn print_knight_attacks() {
        let knight_attacks = KnightAttacks::new();
        // Print attack patterns from different squares for visual verification
        print_bitboard(knight_attacks.0[0], Some(0));    // a1
        print_bitboard(knight_attacks.0[40], Some(40));  // Center
        print_bitboard(knight_attacks.0[17], Some(17));  // Edge
        print_bitboard(knight_attacks.0[55], Some(55));  // Corner
    }
}