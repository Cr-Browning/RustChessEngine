//! Utility functions for chess operations.
//! 
//! This module provides various utility functions for chess operations,
//! including bitboard manipulation, string parsing, and debugging tools.

/// Type alias for a 64-bit integer representing a chess board
pub type Bitboard = u64;

/// Sets a bit in a bitboard based on chess board coordinates.
/// 
/// This function takes chess board coordinates (1-8 for both row and column)
/// and sets the corresponding bit in the bitboard. If the coordinates are
/// outside the valid range, the original bitboard is returned unchanged.
/// 
/// # Arguments
/// 
/// * `row` - The row number (1-8)
/// * `col` - The column number (1-8)
/// 
/// # Returns
/// 
/// * A bitboard with the specified bit set
pub fn set_bit(row: i32, col: i32) -> Bitboard {
    if row < 1 || row > 8 || col < 1 || col > 8 {
        return 0;
    }
    let bit_index = (col - 1) + (row - 1) * 8;
    1 << bit_index
}

/// Splits a string on the first occurrence of a delimiter.
/// 
/// # Arguments
/// 
/// * `s` - The string to split
/// * `delimiter` - The character to split on
/// 
/// # Returns
/// 
/// * A tuple containing the part before the delimiter and the part after
pub fn split_on(s: &str, delimiter: char) -> (&str, &str) {
    match s.find(delimiter) {
        None => (s, ""),
        Some(index) => (&s[..index], &s[index + 1..]),
    }
}

/// Finds the index of the least significant set bit in a bitboard.
/// 
/// This function is used to find the square number of a piece on a bitboard.
/// For example, if a piece is on square e4 (square 28), the bitboard would
/// have bit 28 set, and this function would return 28.
/// 
/// # Arguments
/// 
/// * `bitboard` - The bitboard to scan
/// 
/// # Returns
/// 
/// * The index of the least significant set bit
pub fn bit_scan(bitboard: Bitboard) -> usize {
    match bit_scan_safe(bitboard) {
        Some(index) => index,
        None => 0  // Return 0 for empty bitboards instead of panicking
    }
}

/// Safe version of bit_scan that returns an Option
pub fn bit_scan_safe(bitboard: Bitboard) -> Option<usize> {
    if bitboard == 0 {
        None
    } else {
        Some(bitboard.trailing_zeros() as usize)
    }
}

/// Finds the index of the most significant set bit in a bitboard.
/// 
/// This function is used to find the highest square number of a piece on a bitboard.
/// For example, if pieces are on squares e4 and f6, this function would return
/// the index of f6 as it's the higher square.
/// 
/// # Arguments
/// 
/// * `bitboard` - The bitboard to scan
/// 
/// # Returns
/// 
/// * The index of the most significant set bit
pub fn bit_scan_backward(bitboard: Bitboard) -> usize {
    debug_assert_ne!(bitboard, 0, "Attempted to scan empty bitboard");
    (63 - bitboard.leading_zeros()) as usize
}

/// Extracts all set bits from a bitboard into a vector.
/// 
/// This function is useful when you need to process all pieces or squares
/// represented by a bitboard. It returns a vector of square indices where
/// bits are set.
/// 
/// # Arguments
/// 
/// * `bitboard` - The bitboard to extract bits from
/// 
/// # Returns
/// 
/// * A vector containing the indices of all set bits
pub fn extract_bits(mut bitboard: Bitboard) -> Vec<usize> {
    let mut bits = Vec::new();
    while bitboard != 0 {
        let lsb = bit_scan(bitboard);
        bits.push(lsb);
        bitboard &= !(1 << lsb);
    }
    bits
}

/// Prints a visual representation of a bitboard for debugging.
/// 
/// This function prints a bitboard as an 8x8 grid of 1s and 0s, with an
/// optional highlight for a specific square. This is useful for debugging
/// move generation and position evaluation.
/// 
/// # Arguments
/// 
/// * `bitboard` - The bitboard to print
/// * `highlight` - Optional square index to highlight in the output
pub fn print_bitboard(bitboard: Bitboard, highlight: Option<usize>) {
    println!("Bitboard: {}", bitboard);
    for rank in (0..8).rev() {
        for file in 0..8 {
            let square = rank * 8 + file;
            let bit = (bitboard >> square) & 1;
            
            if let Some(h) = highlight {
                if h == square {
                    print!("\x1b[93m{}\x1b[0m ", bit);
                    continue;
                }
            }
            print!("{} ", bit);
        }
        println!();
    }
    println!();
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Tests that split_on correctly handles space-separated strings
    #[test]
    fn split_on_space_works() {
        let test_string = "A B C D";
        let (should_be_a, rest) = split_on(test_string, ' ');
        assert_eq!(should_be_a, "A");
        assert_eq!(rest, "B C D");
    }

    /// Tests that split_on works with all ASCII characters
    #[test]
    fn split_on_ascii_works() {
        for i in 0..128 {
            let ch = char::from(i);
            if ch == 'A' {
                continue;
            }
            let test_string = format!("AA{}BB{}CC{}DD", ch, ch, ch);
            let (should_be_a, rest) = split_on(&test_string, ch);
            assert_eq!(should_be_a, "AA", "{},{}, {}", test_string, ch, i);
            assert_eq!(rest, &format!("BB{}CC{}DD", ch, ch));
        }
    }

    /// Tests that bit_scan correctly identifies the least significant set bit
    #[test]
    fn bit_scan_works() {
        for i in 0..64 {
            let bit = (1 as u64) << i;
            let index = bit_scan(bit);

            assert_eq!(i, index);
        }
    }

    /// Tests that bit_scan works with multiple set bits
    #[test]
    fn test_bit_scan_with_mult_bits() {
        for lowest_bit in 0..64 {
            let mut bit = 1 << lowest_bit;
            for other_bit in (lowest_bit + 1)..64 {
                bit |= 1 << other_bit;
                let index = bit_scan(bit);
                assert_eq!(lowest_bit, index);
            }
        }
    }
}
