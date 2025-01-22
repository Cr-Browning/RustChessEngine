//! Pawn move and attack pattern generation module.
//! 
//! This module handles the generation of pawn moves and attacks using bitboards.
//! It pre-computes all possible pawn moves and attacks from each square for both
//! white and black pawns, including forward moves and diagonal captures.

use crate::utils::*;
use crate::position::Color;

/// Type alias for a 64-bit integer representing a chess board
type Bitboard = u64;

/// A structure containing pre-computed pawn move and attack patterns.
/// 
/// This struct stores vectors of bitboards representing possible pawn moves
/// and attacks for both white and black pawns from each square. It separates
/// forward moves from diagonal capture moves for efficient move generation.
#[derive(Debug, Clone)]
pub struct PawnAttacks {
    /// Forward moves for white pawns from each square
    pub white_forward_moves: Vec<Bitboard>,
    /// Diagonal capture moves for white pawns from each square
    pub white_diagonal_moves: Vec<Bitboard>,
    /// Forward moves for black pawns from each square
    pub black_forward_moves: Vec<Bitboard>,
    /// Diagonal capture moves for black pawns from each square
    pub black_diagonal_moves: Vec<Bitboard>,
}

impl PawnAttacks {
    /// Creates a new instance with pre-computed pawn move and attack patterns.
    /// 
    /// This function initializes move and attack patterns for all 64 squares
    /// on the board, for both white and black pawns. The patterns are stored
    /// in vectors for efficient lookup during move generation.
    /// 
    /// # Returns
    /// 
    /// * A new `PawnAttacks` instance with all patterns pre-computed
    pub fn new() -> Self {
        let mut w_forward = Vec::with_capacity(64);
        let mut w_diagonal = Vec::with_capacity(64);
        let mut b_forward = Vec::with_capacity(64);
        let mut b_diagonal = Vec::with_capacity(64);

        for square in 0..64 {
            let row = (square / 8 + 1) as i32;
            let col = (square % 8 + 1) as i32;
            
            w_forward.push(forward_move(row, col, Color::White));
            w_diagonal.push(diagonal_move(row, col, Color::White));
            b_forward.push(forward_move(row, col, Color::Black));
            b_diagonal.push(diagonal_move(row, col, Color::Black));
        }

        Self {
            white_forward_moves: w_forward,
            white_diagonal_moves: w_diagonal,
            black_forward_moves: b_forward,
            black_diagonal_moves: b_diagonal,
        }
    }
}

/// Generates a bitboard of forward pawn moves from a given square.
/// 
/// This function calculates possible forward moves for a pawn, including:
/// - Single square advance
/// - Double square advance from starting position (2nd rank for white, 7th for black)
/// 
/// # Arguments
/// 
/// * `row` - The row number (1-8) of the pawn's position
/// * `col` - The column number (1-8) of the pawn's position
/// * `color` - The color of the pawn (White or Black)
/// 
/// # Returns
/// 
/// * A bitboard representing possible forward moves
fn forward_move(row: i32, col: i32, color: Color) -> Bitboard {
    if row == 1 || row == 8 {
        return 0;
    }
    let mut bitboard = 0;
    if color == Color::White {
        if row < 8 {
            bitboard |= set_bit(row + 1, col);
        }
        if row == 2 {
            bitboard |= set_bit(row + 2, col);
        } 
    } else {
        if row > 1 {
            bitboard |= set_bit(row - 1, col);
        }
        if row == 7 {
            bitboard |= set_bit(row - 2, col);
        }
    }
    bitboard
}

/// Generates a bitboard of diagonal pawn captures from a given square.
/// 
/// This function calculates possible diagonal capture moves for a pawn,
/// which can also be used for en passant captures.
/// 
/// # Arguments
/// 
/// * `row` - The row number (1-8) of the pawn's position
/// * `col` - The column number (1-8) of the pawn's position
/// * `color` - The color of the pawn (White or Black)
/// 
/// # Returns
/// 
/// * A bitboard representing possible diagonal capture moves
fn diagonal_move(row: i32, col: i32, color: Color) -> Bitboard {
    if row == 1 || row == 8 {
        return 0;
    }
    let mut bitboard = 0;
    if color == Color::White {
        if row < 8 {
            if col < 8 {  // Only add right diagonal if not on h-file
                bitboard |= set_bit(row + 1, col + 1);
            }
            if col > 1 {  // Only add left diagonal if not on a-file
                bitboard |= set_bit(row + 1, col - 1);
            }
        }
    } else {
        if row > 1 {
            if col < 8 {  // Only add right diagonal if not on h-file
                bitboard |= set_bit(row - 1, col + 1);
            }
            if col > 1 {  // Only add left diagonal if not on a-file
                bitboard |= set_bit(row - 1, col - 1);
            }
        }
    }
    bitboard
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Tests forward moves for white pawns on the second rank
    #[test]
    fn test_second_row_white_pawn() {
        let row = 2;
        for col in 1..=8 {
            let bitboard = forward_move(row, col, Color::White);
            let lsb = bit_scan(bitboard);
            let msb = bit_scan_backward(bitboard);

            let expected_lsb = (col - 1) + (row + 1 - 1) * 8;
            let expected_msb = (col - 1) + (row + 2 - 1) * 8;
            assert_eq!(lsb, expected_lsb as usize);
            assert_eq!(msb, expected_msb as usize);
        }
    }

    /// Tests forward moves for black pawns on the second rank
    #[test]
    fn test_second_row_black_pawn() {
        let row = 2;
        for col in 1..=8 {
            let bitboard = forward_move(row, col, Color::Black);
            let lsb = bit_scan(bitboard);

            let expected_lsb = (col - 1) + (row - 1 - 1) * 8;
            assert_eq!(lsb, expected_lsb as usize);
        }
    }

    /// Tests forward moves for white pawns in middle ranks
    #[test]
    fn test_middle_rows_white_pawn() {
        for row in 3..=7 {
            for col in 1..=8 {
                let bitboard = forward_move(row, col, Color::White);
                let lsb = bit_scan(bitboard);
                let msb = bit_scan_backward(bitboard);

                let expected_lsb = (col - 1) + (row + 1 - 1) * 8;
                assert_eq!(lsb, expected_lsb as usize);
            }
        }
    }

    /// Tests forward moves for black pawns in middle ranks
    #[test]
    fn test_middle_rows_black_pawn() {
        for row in 2..=6 {
            for col in 1..=8 {
                let bitboard = forward_move(row, col, Color::Black);
                let lsb = bit_scan(bitboard);

                let expected_lsb = (col - 1) + (row - 1 - 1) * 8;
                assert_eq!(lsb, expected_lsb as usize);
            }
        }
    }

    /// Tests that pawns on edge ranks cannot move
    #[test]
    fn test_edges() {
        for color in [Color::White, Color::Black] {
            for row in [1, 8] {
                for col in 1..=8 {
                    let bitboard = forward_move(row, col, color);
                    assert_eq!(bitboard, 0);
                }
            }
        }
    }

    /// Tests diagonal capture moves for white pawns
    #[test]
    fn test_diagonal_white() {
        for row in 2..=7 {
            for col in 2..=7 { 
                let bitboard = diagonal_move(row, col, Color::White);
                let lsb = bit_scan(bitboard);
                let msb = bit_scan_backward(bitboard);

                let expected_lsb = (col - 1 - 1) + (row + 1 - 1) * 8;
                let expected_msb = (col + 1 - 1) + (row + 1 - 1) * 8;

                assert_eq!(lsb, expected_lsb as usize);
                assert_eq!(msb, expected_msb as usize);
            }
        }
    }

    /// Tests diagonal capture moves for black pawns
    #[test]
    fn test_diagonal_black() {
        for row in 2..=7 {
            for col in 2..=7 { 
                let bitboard = diagonal_move(row, col, Color::Black);
                let lsb = bit_scan(bitboard);
                let msb = bit_scan_backward(bitboard);

                let expected_lsb = (col - 1 - 1) + (row - 1 - 1) * 8;
                let expected_msb = (col + 1 - 1) + (row - 1 - 1) * 8;

                assert_eq!(lsb, expected_lsb as usize);
                assert_eq!(msb, expected_msb as usize);
            }
        }
    }

    /// Tests diagonal capture moves for white pawns on board edges
    #[test]
    fn test_diagonal_edge_white() {
        for row in 2..=7 {
            let col = 1;
            let bitboard = diagonal_move(row, col, Color::White);
            let lsb = bit_scan(bitboard);

            let expected_lsb = (col + 1 - 1) + (row - 1 + 1) * 8;
            assert_eq!(lsb, expected_lsb as usize);

            let col = 8;
            let bitboard = diagonal_move(row, col, Color::White);
            let lsb = bit_scan(bitboard);

            let expected_lsb = (col - 1 - 1) + (row - 1 + 1) * 8;
            assert_eq!(lsb, expected_lsb as usize);
        }
    }

    /// Tests diagonal capture moves for black pawns on board edges
    #[test]
    fn test_diagonal_edge_black() {
        for row in 2..=7 {
            let col = 1;
            let bitboard = diagonal_move(row, col, Color::Black);
            let lsb = bit_scan(bitboard);

            let expected_lsb = (col + 1 - 1) + (row - 1 - 1) * 8;
            assert_eq!(lsb, expected_lsb as usize);

            let col = 8;
            let bitboard = diagonal_move(row, col, Color::Black);
            let lsb = bit_scan(bitboard);

            let expected_lsb = (col - 1 - 1) + (row - 1 - 1) * 8;
            assert_eq!(lsb, expected_lsb as usize);
        }
    }

    /// Tests that PawnAttacks can be initialized without panicking
    #[test]
    fn test_pawnattacks_init() {
        let pawnattacks = PawnAttacks::new();
    }
}
