//! Ray attack generation module for sliding pieces.
//! 
//! This module handles the generation of ray attacks for sliding pieces (bishops, rooks, and queens)
//! using efficient bitboard operations. It pre-computes ray attacks in all eight directions and
//! provides methods to calculate attacks considering blocking pieces.

use crate::utils::*;

/// Type alias for a 64-bit integer representing a chess board
type Bitboard = u64;

/// A structure containing pre-computed ray attacks in all eight directions.
/// 
/// This struct stores vectors of bitboards representing ray attacks from each square
/// in all eight possible directions (N, E, S, W, NE, SE, NW, SW). These rays are used
/// to efficiently calculate sliding piece moves.
#[derive(Debug, Clone)]
pub struct Rays {
    /// North-directed rays from each square
    pub n_rays: Vec<Bitboard>,
    /// East-directed rays from each square
    pub e_rays: Vec<Bitboard>,
    /// South-directed rays from each square
    pub s_rays: Vec<Bitboard>,
    /// West-directed rays from each square
    pub w_rays: Vec<Bitboard>,
    /// Northeast-directed rays from each square
    pub ne_rays: Vec<Bitboard>,
    /// Southeast-directed rays from each square
    pub se_rays: Vec<Bitboard>,
    /// Northwest-directed rays from each square
    pub nw_rays: Vec<Bitboard>,
    /// Southwest-directed rays from each square
    pub sw_rays: Vec<Bitboard>,
}

impl Rays {
    /// Creates a new instance with pre-computed ray attacks for all squares.
    /// 
    /// This function initializes ray attacks in all eight directions for each square
    /// on the board. The rays are stored in vectors for efficient lookup during move
    /// generation.
    /// 
    /// # Returns
    /// 
    /// * A new `Rays` instance with all ray attacks pre-computed
    pub fn new() -> Self {
        // Pre-calculate all rays at initialization
        let mut rays = Self {
            n_rays: Vec::with_capacity(64),
            e_rays: Vec::with_capacity(64),
            s_rays: Vec::with_capacity(64),
            w_rays: Vec::with_capacity(64),
            ne_rays: Vec::with_capacity(64),
            se_rays: Vec::with_capacity(64),
            nw_rays: Vec::with_capacity(64),
            sw_rays: Vec::with_capacity(64),
        };
        
        for square in 0..64 {
            let row = (square / 8 + 1) as i64;
            let col = (square % 8 + 1) as i64;
            rays.n_rays.push(n_ray(row, col));
            rays.e_rays.push(e_ray(row, col));
            rays.s_rays.push(s_ray(row, col));
            rays.w_rays.push(w_ray(row, col));
            rays.ne_rays.push(ne_ray(row, col));
            rays.se_rays.push(se_ray(row, col));
            rays.nw_rays.push(nw_ray(row, col));
            rays.sw_rays.push(sw_ray(row, col));
        }
        rays
    }

    /// Calculates bishop attacks from a given square considering occupied squares.
    /// 
    /// This function combines diagonal ray attacks (NE, SE, NW, SW) and handles blocking
    /// pieces to determine valid bishop moves.
    /// 
    /// # Arguments
    /// 
    /// * `square` - The square index (0-63) from which to generate attacks
    /// * `occupancy` - A bitboard representing all occupied squares
    /// 
    /// # Returns
    /// 
    /// * A bitboard representing all squares the bishop can attack
    pub fn get_bishop_attacks(&self, square: usize, occupancy: Bitboard) -> Bitboard {
        let mut attacks = 0;
        
        // Northeast ray
        let ne = self.ne_rays[square];
        let blockers = ne & occupancy;
        if blockers != 0 {
            let blocker_square = bit_scan(blockers);
            if blocker_square < 63 {  // Prevent overflow
                attacks |= ne & ((1u64 << (blocker_square + 1)) - 1);
            } else {
                attacks |= ne & !((1u64 << blocker_square) - 1);
            }
        } else {
            attacks |= ne;
        }

        // Northwest ray
        let nw = self.nw_rays[square];
        let blockers = nw & occupancy;
        if blockers != 0 {
            let blocker_square = bit_scan(blockers);
            if blocker_square < 63 {  // Prevent overflow
                attacks |= nw & ((1u64 << (blocker_square + 1)) - 1);
            } else {
                attacks |= nw & !((1u64 << blocker_square) - 1);
            }
        } else {
            attacks |= nw;
        }

        // Southeast ray
        let se = self.se_rays[square];
        let blockers = se & occupancy;
        if blockers != 0 {
            let blocker_square = bit_scan_backward(blockers);
            attacks |= se & !((1u64 << blocker_square) - 1) | (1u64 << blocker_square);
        } else {
            attacks |= se;
        }

        // Southwest ray
        let sw = self.sw_rays[square];
        let blockers = sw & occupancy;
        if blockers != 0 {
            let blocker_square = bit_scan_backward(blockers);
            attacks |= sw & !((1u64 << blocker_square) - 1) | (1u64 << blocker_square);
        } else {
            attacks |= sw;
        }

        attacks
    }

    /// Calculates rook attacks from a given square considering occupied squares.
    /// 
    /// This function combines orthogonal ray attacks (N, E, S, W) and handles blocking
    /// pieces to determine valid rook moves.
    /// 
    /// # Arguments
    /// 
    /// * `square` - The square index (0-63) from which to generate attacks
    /// * `occupancy` - A bitboard representing all occupied squares
    /// 
    /// # Returns
    /// 
    /// * A bitboard representing all squares the rook can attack
    pub fn get_rook_attacks(&self, square: usize, occupancy: Bitboard) -> Bitboard {
        let mut attacks = 0;
        
        // North ray
        let north = self.n_rays[square];
        let blockers = north & occupancy;
        if blockers != 0 {
            let blocker_square = bit_scan(blockers);
            if blocker_square < 63 {  // Prevent overflow
                attacks |= (north & ((1u64 << (blocker_square + 1)) - 1)) | (1u64 << blocker_square);
            } else {
                attacks |= north & !((1u64 << blocker_square) - 1);
            }
        } else {
            attacks |= north;
        }

        // South ray
        let south = self.s_rays[square];
        let blockers = south & occupancy;
        if blockers != 0 {
            let blocker_square = bit_scan_backward(blockers);
            attacks |= (south & !((1u64 << blocker_square) - 1)) | (1u64 << blocker_square);
        } else {
            attacks |= south;
        }

        // East ray
        let east = self.e_rays[square];
        let blockers = east & occupancy;
        if blockers != 0 {
            let blocker_square = bit_scan(blockers);
            if blocker_square < 63 {  // Prevent overflow
                attacks |= (east & ((1u64 << (blocker_square + 1)) - 1)) | (1u64 << blocker_square);
            } else {
                attacks |= east & !((1u64 << blocker_square) - 1);
            }
        } else {
            attacks |= east;
        }

        // West ray
        let west = self.w_rays[square];
        let blockers = west & occupancy;
        if blockers != 0 {
            let blocker_square = bit_scan_backward(blockers);
            attacks |= (west & !((1u64 << blocker_square) - 1)) | (1u64 << blocker_square);
        } else {
            attacks |= west;
        }

        attacks
    }

    /// Calculates queen attacks from a given square considering occupied squares.
    /// 
    /// This function combines bishop and rook attacks since a queen can move in
    /// both diagonal and orthogonal directions.
    /// 
    /// # Arguments
    /// 
    /// * `square` - The square index (0-63) from which to generate attacks
    /// * `occupancy` - A bitboard representing all occupied squares
    /// 
    /// # Returns
    /// 
    /// * A bitboard representing all squares the queen can attack
    pub fn get_queen_attacks(&self, square: usize, occupancy: Bitboard) -> Bitboard {
        self.get_bishop_attacks(square, occupancy) | self.get_rook_attacks(square, occupancy)
    }
}

/// Macro for generating ray attack functions.
/// 
/// This macro creates functions that generate ray attacks in a specific direction
/// based on the provided offset function.
/// 
/// # Arguments
/// 
/// * `name` - The name of the ray generation function to create
/// * `offset_fn` - A closure that calculates the next square in the ray's direction
macro_rules! define_ray {
    ($name:ident, $offset_fn:expr) =>{
        fn $name(row: i64, col: i64) -> Bitboard {
            let mut bitboard = 0;
            for offset in 1..=8 {
                bitboard = set_bit(bitboard, $offset_fn(row, col, offset));
            }
            bitboard
         }
    };
}

// Define ray generation functions for all eight directions
define_ray!(n_ray, |row, col, offset| (row + offset, col));
define_ray!(e_ray, |row, col, offset| (row, col + offset));
define_ray!(s_ray, |row, col, offset| (row - offset, col));
define_ray!(w_ray, |row, col, offset| (row, col - offset));
define_ray!(ne_ray, |row, col, offset| (row + offset, col + offset));
define_ray!(nw_ray, |row, col, offset| (row + offset, col - offset));
define_ray!(se_ray, |row, col, offset| (row - offset, col + offset));
define_ray!(sw_ray, |row, col, offset| (row - offset, col - offset));

/// Sets a bit in a bitboard based on chess board coordinates.
/// 
/// # Arguments
/// 
/// * `bitboard` - The bitboard to modify
/// * `row_col` - A tuple containing (row, column) coordinates (1-8, 1-8)
/// 
/// # Returns
/// 
/// * The modified bitboard with the bit set at the specified position
fn set_bit(bitboard: Bitboard, row_col: (i64, i64)) -> Bitboard {
    let row = row_col.0;
    let col = row_col.1;
    if row < 1 || row > 8 || col < 1 || col > 8 {
        return bitboard;
    }
    bitboard | (1 << ((col -1) + (row - 1) * 8))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Tests bishop attack generation with a blocking piece
    #[test]
    fn test_bishop_attacks() {
        let rays = Rays::new();
        let occupancy = 1u64 << 35; // Place a piece in the middle of the board
        let attacks = rays.get_bishop_attacks(28, occupancy); // Test from e4
        assert!(attacks & occupancy != 0); // Should be able to capture the piece
    }

    /// Tests rook attack generation with a blocking piece
    #[test]
    fn test_rook_attacks() {
        let rays = Rays::new();
        let occupancy = 1u64 << 36; // Place a piece in the middle of the board
        let attacks = rays.get_rook_attacks(28, occupancy); // Test from e4
        assert!(attacks & occupancy != 0); // Should be able to capture the piece
    }

    /// Tests queen attack generation with multiple blocking pieces
    #[test]
    fn test_queen_attacks() {
        let rays = Rays::new();
        let occupancy = (1u64 << 35) | (1u64 << 36); // Place pieces diagonally and orthogonally
        let attacks = rays.get_queen_attacks(28, occupancy); // Test from e4
        assert!(attacks & occupancy == occupancy); // Should be able to capture both pieces
    }
}