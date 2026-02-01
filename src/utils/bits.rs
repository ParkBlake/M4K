//! Bit manipulation utilities
//!
//! This module provides various bit manipulation functions that are useful
//! throughout the chess engine.

use crate::bitboard::Bitboard;

/// Count trailing zeros in a u64
#[inline(always)]
pub fn trailing_zeros(x: u64) -> u32 {
    if x == 0 {
        64
    } else {
        x.trailing_zeros()
    }
}

/// Count leading zeros in a u64
#[inline(always)]
pub fn leading_zeros(x: u64) -> u32 {
    if x == 0 {
        64
    } else {
        x.leading_zeros()
    }
}

/// Find the index of the least significant set bit
#[inline(always)]
pub fn lsb_index(x: u64) -> u32 {
    trailing_zeros(x)
}

/// Find the index of the most significant set bit
#[inline(always)]
pub fn msb_index(x: u64) -> u32 {
    63 - leading_zeros(x)
}

/// Check if a number is a power of 2
#[inline(always)]
pub fn is_power_of_two(x: u64) -> bool {
    x != 0 && (x & (x - 1)) == 0
}

/// Get the next power of 2 greater than or equal to x
#[inline(always)]
pub fn next_power_of_two(x: u64) -> u64 {
    if x == 0 {
        1
    } else if is_power_of_two(x) {
        x
    } else {
        1 << (64 - leading_zeros(x - 1))
    }
}

/// Reverse the bits of a u64
#[inline(always)]
pub fn reverse_bits(x: u64) -> u64 {
    x.reverse_bits()
}

/// Swap bytes in a u64 (endianness conversion)
#[inline(always)]
pub fn swap_bytes(x: u64) -> u64 {
    x.swap_bytes()
}

/// Rotate left by n bits
#[inline(always)]
pub fn rotate_left(x: u64, n: u32) -> u64 {
    x.rotate_left(n)
}

/// Rotate right by n bits
#[inline(always)]
pub fn rotate_right(x: u64, n: u32) -> u64 {
    x.rotate_right(n)
}

/// Extract bits from position 'start' to 'end' (inclusive)
#[inline(always)]
pub fn extract_bits(x: u64, start: u32, end: u32) -> u64 {
    let mask = if end >= 63 {
        u64::MAX
    } else {
        (1 << (end + 1)) - 1
    };
    (x & mask) >> start
}

/// Deposit bits into a u64 at position 'start'
#[inline(always)]
pub fn deposit_bits(value: u64, bits: u64, start: u32) -> u64 {
    let mask = !(u64::MAX << bits.count_ones()) << start;
    (value & !mask) | ((bits << start) & mask)
}

/// Count the number of set bits (population count)
#[inline(always)]
pub fn popcount(x: u64) -> u32 {
    x.count_ones()
}

/// Check if a specific bit is set
#[inline(always)]
pub fn test_bit(x: u64, bit: u32) -> bool {
    (x & (1 << bit)) != 0
}

/// Set a specific bit
#[inline(always)]
pub fn set_bit(x: u64, bit: u32) -> u64 {
    x | (1 << bit)
}

/// Clear a specific bit
#[inline(always)]
pub fn clear_bit(x: u64, bit: u32) -> u64 {
    x & !(1 << bit)
}

/// Toggle a specific bit
#[inline(always)]
pub fn toggle_bit(x: u64, bit: u32) -> u64 {
    x ^ (1 << bit)
}

/// Find first set bit and clear it
#[inline(always)]
pub fn find_and_clear_lsb(x: &mut u64) -> Option<u32> {
    if *x == 0 {
        None
    } else {
        let bit = lsb_index(*x);
        *x &= *x - 1;
        Some(bit)
    }
}

/// Interleave bits from two u32 values to create a u64 (for Morton codes)
#[inline(always)]
pub fn interleave_bits(x: u32, y: u32) -> u64 {
    let mut result = 0u64;
    let mut mask = 1u64;

    for i in 0..32 {
        if (x & (1 << i)) != 0 {
            result |= mask;
        }
        mask <<= 1;
        if (y & (1 << i)) != 0 {
            result |= mask;
        }
        mask <<= 1;
    }

    result
}

/// Deinterleave bits from a u64 Morton code back to two u32 values
#[inline(always)]
pub fn deinterleave_bits(z: u64) -> (u32, u32) {
    let mut x = 0u32;
    let mut y = 0u32;

    let mut mask = 1u64;
    for i in 0..32 {
        if (z & mask) != 0 {
            x |= 1 << i;
        }
        mask <<= 1;
        if (z & mask) != 0 {
            y |= 1 << i;
        }
        mask <<= 1;
    }

    (x, y)
}

/// Calculate the Hamming distance between two u64 values
#[inline(always)]
pub fn hamming_distance(a: u64, b: u64) -> u32 {
    (a ^ b).count_ones()
}

/// Check if two bitboards have any bits in common
#[inline(always)]
pub fn has_intersection(a: Bitboard, b: Bitboard) -> bool {
    (a.0 & b.0) != 0
}

/// Get the intersection of two bitboards
#[inline(always)]
pub fn intersection(a: Bitboard, b: Bitboard) -> Bitboard {
    Bitboard(a.0 & b.0)
}

/// Get the union of two bitboards
#[inline(always)]
pub fn union(a: Bitboard, b: Bitboard) -> Bitboard {
    Bitboard(a.0 | b.0)
}

/// Get the symmetric difference of two bitboards
#[inline(always)]
pub fn symmetric_difference(a: Bitboard, b: Bitboard) -> Bitboard {
    Bitboard(a.0 ^ b.0)
}

/// Check if a bitboard is a subset of another
#[inline(always)]
pub fn is_subset(subset: Bitboard, superset: Bitboard) -> bool {
    (subset.0 & superset.0) == subset.0
}

/// Check if a bitboard is empty
#[inline(always)]
pub fn is_empty(bb: Bitboard) -> bool {
    bb.0 == 0
}

/// Get the complement of a bitboard (flip all bits)
#[inline(always)]
pub fn complement(bb: Bitboard) -> Bitboard {
    Bitboard(!bb.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trailing_zeros() {
        assert_eq!(trailing_zeros(0), 64);
        assert_eq!(trailing_zeros(1), 0);
        assert_eq!(trailing_zeros(2), 1);
        assert_eq!(trailing_zeros(0xFF00), 8);
    }

    #[test]
    fn test_leading_zeros() {
        assert_eq!(leading_zeros(0), 64);
        assert_eq!(leading_zeros(1), 63);
        assert_eq!(leading_zeros(0xFF00_0000_0000_0000), 0);
    }

    #[test]
    fn test_power_of_two() {
        assert!(is_power_of_two(1));
        assert!(is_power_of_two(2));
        assert!(is_power_of_two(4));
        assert!(is_power_of_two(1024));
        assert!(!is_power_of_two(3));
        assert!(!is_power_of_two(6));
        assert!(!is_power_of_two(0));
    }

    #[test]
    fn test_next_power_of_two() {
        assert_eq!(next_power_of_two(0), 1);
        assert_eq!(next_power_of_two(1), 1);
        assert_eq!(next_power_of_two(2), 2);
        assert_eq!(next_power_of_two(3), 4);
        assert_eq!(next_power_of_two(5), 8);
        assert_eq!(next_power_of_two(17), 32);
    }

    #[test]
    fn test_bit_operations() {
        let mut x = 0u64;

        x = set_bit(x, 5);
        assert!(test_bit(x, 5));
        assert_eq!(x, 32);

        x = set_bit(x, 10);
        assert!(test_bit(x, 10));
        assert_eq!(x, 32 | 1024);

        x = clear_bit(x, 5);
        assert!(!test_bit(x, 5));
        assert!(test_bit(x, 10));

        x = toggle_bit(x, 5);
        assert!(test_bit(x, 5));
    }

    #[test]
    fn test_find_and_clear_lsb() {
        let mut x = 0b1011_0000u64;
        assert_eq!(find_and_clear_lsb(&mut x), Some(4));
        assert_eq!(x, 0b1010_0000u64);

        assert_eq!(find_and_clear_lsb(&mut x), Some(5));
        assert_eq!(x, 0b1000_0000u64);

        assert_eq!(find_and_clear_lsb(&mut x), Some(7));
        assert_eq!(x, 0u64);

        assert_eq!(find_and_clear_lsb(&mut x), None);
    }

    #[test]
    fn test_interleave_bits() {
        let x = 0b1010u32;
        let y = 0b1100u32;
        let z = interleave_bits(x, y);
        assert_eq!(z, 0b11001010u64);

        let (dx, dy) = deinterleave_bits(z);
        assert_eq!(dx, x);
        assert_eq!(dy, y);
    }

    #[test]
    fn test_hamming_distance() {
        assert_eq!(hamming_distance(0b1010, 0b1010), 0);
        assert_eq!(hamming_distance(0b1010, 0b0101), 4);
        assert_eq!(hamming_distance(0b1111, 0b0000), 4);
    }

    #[test]
    fn test_bitboard_operations() {
        let a = Bitboard(0b1010);
        let b = Bitboard(0b1100);

        assert!(has_intersection(a, b));
        assert_eq!(intersection(a, b), Bitboard(0b1000));
        assert_eq!(union(a, b), Bitboard(0b1110));
        assert_eq!(symmetric_difference(a, b), Bitboard(0b0110));

        assert!(is_subset(Bitboard(0b1010), Bitboard(0b1111)));
        assert!(!is_subset(Bitboard(0b1010), Bitboard(0b0101)));

        assert!(!is_empty(Bitboard(0b1010)));
        assert!(is_empty(Bitboard(0)));
    }
}
