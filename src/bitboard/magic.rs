//! Magic bitboard implementation for sliding piece attack generation
//!
//! This module provides highly optimized attack generation for bishops and rooks
//! using magic bitboards, which are faster than classical approaches.

use super::types::*;
use crate::asm::prelude::*;

/// Magic number entry for a square
#[derive(Clone, Copy)]
struct MagicEntry {
    /// The magic number
    magic: u64,
    /// Mask of relevant occupancy bits
    mask: Bitboard,
    /// Shift amount
    shift: u32,
    /// Offset into the attack table
    offset: usize,
}

/// Precomputed magic entries for bishops
static BISHOP_MAGICS: [MagicEntry; 64] = generate_bishop_magics();

/// Precomputed magic entries for rooks
static ROOK_MAGICS: [MagicEntry; 64] = generate_rook_magics();

/// Attack tables for bishops and rooks
/// Size is large enough to hold all possible attacks
static mut BISHOP_ATTACKS: [Bitboard; 5248] = [Bitboard::EMPTY; 5248];
static mut ROOK_ATTACKS: [Bitboard; 102400] = [Bitboard::EMPTY; 102400];

/// Initialize magic bitboard tables
/// This must be called before using magic bitboard functions
pub fn init_magics() {
    unsafe {
        init_bishop_attacks();
        init_rook_attacks();
    }
}

/// Generate bishop magic entries
const fn generate_bishop_magics() -> [MagicEntry; 64] {
    // Placeholder magic numbers - in a real implementation, these would be
    // carefully chosen to minimize collisions and table size
    let mut magics = [MagicEntry {
        magic: 0,
        mask: Bitboard::EMPTY,
        shift: 0,
        offset: 0,
    }; 64];

    let mut sq = 0;
    while sq < 64 {
        // Generate bishop-relevant occupancy mask
        let mask = bishop_relevant_mask(Square(sq as u8));

        // Placeholder magic number (would be precomputed)
        let magic = 0x100020410080400; // Example magic

        magics[sq] = MagicEntry {
            magic,
            mask,
            shift: mask.count() as u32,
            offset: 0, // Will be set during initialization
        };

        sq += 1;
    }

    magics
}

/// Generate rook magic entries
const fn generate_rook_magics() -> [MagicEntry; 64] {
    let mut magics = [MagicEntry {
        magic: 0,
        mask: Bitboard::EMPTY,
        shift: 0,
        offset: 0,
    }; 64];

    let mut sq = 0;
    while sq < 64 {
        // Generate rook-relevant occupancy mask
        let mask = rook_relevant_mask(Square(sq as u8));

        // Placeholder magic number
        let magic = 0x8000100040002000; // Example magic

        magics[sq] = MagicEntry {
            magic,
            mask,
            shift: mask.count() as u32,
            offset: 0,
        };

        sq += 1;
    }

    magics
}

/// Get bishop-relevant occupancy mask for a square
fn bishop_relevant_mask(square: Square) -> Bitboard {
    let mut mask = Bitboard::EMPTY;
    let sq = square.0 as i32;
    let rank = sq / 8;
    let file = sq % 8;

    // Generate mask for all squares a bishop could potentially attack
    // (excluding edges that can't be blocked)

    // Up-left
    let mut r = rank + 1;
    let mut f = file + 1;
    while r < 7 && f < 7 {
        mask.set(Square((r * 8 + f) as u8));
        r += 1;
        f += 1;
    }

    // Up-right
    r = rank + 1;
    f = file - 1;
    while r < 7 && f > 0 {
        mask.set(Square((r * 8 + f) as u8));
        r += 1;
        f -= 1;
    }

    // Down-left
    r = rank - 1;
    f = file + 1;
    while r > 0 && f < 7 {
        mask.set(Square((r * 8 + f) as u8));
        r -= 1;
        f += 1;
    }

    // Down-right
    r = rank - 1;
    f = file - 1;
    while r > 0 && f > 0 {
        mask.set(Square((r * 8 + f) as u8));
        r -= 1;
        f -= 1;
    }

    mask
}

/// Get rook-relevant occupancy mask for a square
fn rook_relevant_mask(square: Square) -> Bitboard {
    let mut mask = Bitboard::EMPTY;
    let sq = square.0 as i32;
    let rank = sq / 8;
    let file = sq % 8;

    // Generate mask for all squares a rook could potentially attack
    // (excluding the edges)

    // Up
    let mut r = rank + 1;
    while r < 7 {
        mask.set(Square((r * 8 + file) as u8));
        r += 1;
    }

    // Down
    r = rank - 1;
    while r > 0 {
        mask.set(Square((r * 8 + file) as u8));
        r -= 1;
    }

    // Right
    let mut f = file + 1;
    while f < 7 {
        mask.set(Square((rank * 8 + f) as u8));
        f += 1;
    }

    // Left
    f = file - 1;
    while f > 0 {
        mask.set(Square((rank * 8 + f) as u8));
        f -= 1;
    }

    mask
}

/// Initialize bishop attack tables
unsafe fn init_bishop_attacks() {
    let mut offset = 0;

    for sq in 0..64 {
        let entry = &BISHOP_MAGICS[sq];
        let square = Square(sq as u8);

        // Update offset
        let mut magic_entry = BISHOP_MAGICS[sq];
        magic_entry.offset = offset;
        BISHOP_MAGICS[sq] = magic_entry;

        // Generate all possible subsets of the mask
        let num_subsets = 1u64 << entry.mask.count();
        for subset_idx in 0..num_subsets {
            let occupied = pdep_neon(subset_idx, entry.mask.0);
            let attacks = generate_bishop_attacks_slow(square, Bitboard(occupied));
            let index =
                (occupied.wrapping_mul(entry.magic) >> (64 - entry.shift)) as usize + offset;
            BISHOP_ATTACKS[index] = attacks;
        }

        offset += num_subsets as usize;
    }
}

/// Initialize rook attack tables
unsafe fn init_rook_attacks() {
    let mut offset = 0;

    for sq in 0..64 {
        let entry = &ROOK_MAGICS[sq];
        let square = Square(sq as u8);

        let mut magic_entry = ROOK_MAGICS[sq];
        magic_entry.offset = offset;
        ROOK_MAGICS[sq] = magic_entry;

        let num_subsets = 1u64 << entry.mask.count();
        for subset_idx in 0..num_subsets {
            let occupied = pdep_neon(subset_idx, entry.mask.0);
            let attacks = generate_rook_attacks_slow(square, Bitboard(occupied));
            let index =
                (occupied.wrapping_mul(entry.magic) >> (64 - entry.shift)) as usize + offset;
            ROOK_ATTACKS[index] = attacks;
        }

        offset += num_subsets as usize;
    }
}

/// Slow bishop attack generation for initialization
fn generate_bishop_attacks_slow(square: Square, occupied: Bitboard) -> Bitboard {
    let mut attacks = Bitboard::EMPTY;
    let sq = square.0 as i32;

    // Directions: up-left, up-right, down-left, down-right
    let directions = [(-1, -1), (-1, 1), (1, -1), (1, 1)];

    for &(dr, df) in &directions {
        let mut r = (sq / 8) + dr;
        let mut f = (sq % 8) + df;

        while r >= 0 && r < 8 && f >= 0 && f < 8 {
            let target_sq = (r * 8 + f) as u8;
            attacks.set(Square(target_sq));

            // Stop if we hit a piece
            if occupied.is_occupied(Square(target_sq)) {
                break;
            }

            r += dr;
            f += df;
        }
    }

    attacks
}

/// Slow rook attack generation for initialization
fn generate_rook_attacks_slow(square: Square, occupied: Bitboard) -> Bitboard {
    let mut attacks = Bitboard::EMPTY;
    let sq = square.0 as i32;

    // Directions: up, down, left, right
    let directions = [(-1, 0), (1, 0), (0, -1), (0, 1)];

    for &(dr, df) in &directions {
        let mut r = (sq / 8) + dr;
        let mut f = (sq % 8) + df;

        while r >= 0 && r < 8 && f >= 0 && f < 8 {
            let target_sq = (r * 8 + f) as u8;
            attacks.set(Square(target_sq));

            // Stop if we hit a piece
            if occupied.is_occupied(Square(target_sq)) {
                break;
            }

            r += dr;
            f += df;
        }
    }

    attacks
}

/// Get bishop attacks using magic bitboards
#[inline(always)]
pub fn bishop_attacks_magic(square: Square, occupied: Bitboard) -> Bitboard {
    let entry = &BISHOP_MAGICS[square.0 as usize];
    let relevant_occupied = occupied.0 & entry.mask.0;
    let index =
        (relevant_occupied.wrapping_mul(entry.magic) >> (64 - entry.shift)) as usize + entry.offset;
    unsafe { BISHOP_ATTACKS[index] }
}

/// Get rook attacks using magic bitboards
#[inline(always)]
pub fn rook_attacks_magic(square: Square, occupied: Bitboard) -> Bitboard {
    let entry = &ROOK_MAGICS[square.0 as usize];
    let relevant_occupied = occupied.0 & entry.mask.0;
    let index =
        (relevant_occupied.wrapping_mul(entry.magic) >> (64 - entry.shift)) as usize + entry.offset;
    unsafe { ROOK_ATTACKS[index] }
}

/// Get queen attacks using magic bitboards
#[inline(always)]
pub fn queen_attacks_magic(square: Square, occupied: Bitboard) -> Bitboard {
    bishop_attacks_magic(square, occupied) | rook_attacks_magic(square, occupied)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_magic_initialization() {
        init_magics();
        // Test that tables are populated
        let attacks = bishop_attacks_magic(Square::E4, Bitboard::EMPTY);
        assert!(attacks.count() > 0);
    }

    #[test]
    fn test_bishop_magic_attacks() {
        init_magics();
        let attacks = bishop_attacks_magic(Square::E4, Bitboard::EMPTY);
        // Should attack all diagonals
        assert!(attacks.is_occupied(Square::D3));
        assert!(attacks.is_occupied(Square::F5));
    }

    #[test]
    fn test_rook_magic_attacks() {
        init_magics();
        let attacks = rook_attacks_magic(Square::E4, Bitboard::EMPTY);
        // Should attack entire rank and file
        assert!(attacks.is_occupied(Square::E1));
        assert!(attacks.is_occupied(Square::E8));
        assert!(attacks.is_occupied(Square::A4));
        assert!(attacks.is_occupied(Square::H4));
    }
}
