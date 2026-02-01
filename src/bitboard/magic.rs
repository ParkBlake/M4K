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
static mut BISHOP_MAGICS: [MagicEntry; 64] = [MagicEntry {
    magic: 0,
    mask: Bitboard::EMPTY,
    shift: 0,
    offset: 0,
}; 64];

/// Precomputed magic entries for rooks
static mut ROOK_MAGICS: [MagicEntry; 64] = [MagicEntry {
    magic: 0,
    mask: Bitboard::EMPTY,
    shift: 0,
    offset: 0,
}; 64];

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
    let mut f = file - 1;
    while r < 7 && f > 0 {
        mask.set(Square((r * 8 + f) as u8));
        r += 1;
        f -= 1;
    }

    // Up-right
    r = rank + 1;
    f = file + 1;
    while r < 7 && f < 7 {
        mask.set(Square((r * 8 + f) as u8));
        r += 1;
        f += 1;
    }

    // Down-left
    r = rank - 1;
    f = file - 1;
    while r > 0 && f > 0 {
        mask.set(Square((r * 8 + f) as u8));
        r -= 1;
        f -= 1;
    }

    // Down-right
    r = rank - 1;
    f = file + 1;
    while r > 0 && f < 7 {
        mask.set(Square((r * 8 + f) as u8));
        r -= 1;
        f += 1;
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

/// Find a magic number for a given mask and attack function
fn find_magic(mask: Bitboard, attack_fn: fn(Square, Bitboard) -> Bitboard, square: Square) -> u64 {
    let bits = mask.count();
    let num_subsets = 1u64 << bits;
    let mut attacks = vec![Bitboard::EMPTY; num_subsets as usize];
    let mut occupancies = vec![0u64; num_subsets as usize];

    // Generate all subsets and their attacks
    for i in 0..num_subsets {
        let occupied = pdep_neon(i, mask.0);
        occupancies[i as usize] = occupied;
        attacks[i as usize] = attack_fn(square, Bitboard(occupied));
    }

    // Try random magic numbers until we find one that works
    use std::collections::HashSet;
    loop {
        let magic = rand::random::<u64>() & rand::random::<u64>() & rand::random::<u64>();
        if (magic.wrapping_mul(mask.0)).count_ones() < 6 {
            continue; // Bad magic
        }

        let mut used = HashSet::new();
        let mut ok = true;
        for i in 0..num_subsets {
            let index = (occupancies[i as usize].wrapping_mul(magic) >> (64 - bits)) as usize;
            if used.contains(&index) {
                ok = false;
                break;
            }
            used.insert(index);
        }
        if ok {
            return magic;
        }
    }
}

/// Initialize bishop attack tables
unsafe fn init_bishop_attacks() {
    let mut offset = 0;

    for sq in 0..64 {
        let square = Square(sq as u8);
        let mask = bishop_relevant_mask(square);
        let magic = find_magic(mask, generate_bishop_attacks_slow, square);
        let shift = mask.count() as u32;

        BISHOP_MAGICS[sq] = MagicEntry {
            magic,
            mask,
            shift,
            offset,
        };

        // Generate all possible subsets of the mask
        let num_subsets = 1u64 << mask.count();
        for subset_idx in 0..num_subsets {
            let occupied = pdep_neon(subset_idx, mask.0);
            let attacks = generate_bishop_attacks_slow(square, Bitboard(occupied));
            let index =
                (occupied.wrapping_mul(magic) >> (64 - shift)) as usize + offset;
            BISHOP_ATTACKS[index] = attacks;
        }

        offset += num_subsets as usize;
    }
}

/// Initialize rook attack tables
unsafe fn init_rook_attacks() {
    let mut offset = 0;

    for sq in 0..64 {
        let square = Square(sq as u8);
        let mask = rook_relevant_mask(square);
        let magic = find_magic(mask, generate_rook_attacks_slow, square);
        let shift = mask.count() as u32;

        ROOK_MAGICS[sq] = MagicEntry {
            magic,
            mask,
            shift,
            offset,
        };

        let num_subsets = 1u64 << mask.count();
        for subset_idx in 0..num_subsets {
            let occupied = pdep_neon(subset_idx, mask.0);
            let attacks = generate_rook_attacks_slow(square, Bitboard(occupied));
            let index =
                (occupied.wrapping_mul(magic) >> (64 - shift)) as usize + offset;
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
pub unsafe fn bishop_attacks_magic(square: Square, occupied: Bitboard) -> Bitboard {
    let entry = &BISHOP_MAGICS[square.0 as usize];
    let relevant_occupied = occupied.0 & entry.mask.0;
    let index =
        (relevant_occupied.wrapping_mul(entry.magic) >> (64 - entry.shift)) as usize + entry.offset;
    BISHOP_ATTACKS[index]
}

/// Get rook attacks using magic bitboards
#[inline(always)]
pub unsafe fn rook_attacks_magic(square: Square, occupied: Bitboard) -> Bitboard {
    let entry = &ROOK_MAGICS[square.0 as usize];
    let relevant_occupied = occupied.0 & entry.mask.0;
    let index =
        (relevant_occupied.wrapping_mul(entry.magic) >> (64 - entry.shift)) as usize + entry.offset;
    ROOK_ATTACKS[index]
}

/// Get queen attacks using magic bitboards
#[inline(always)]
pub unsafe fn queen_attacks_magic(square: Square, occupied: Bitboard) -> Bitboard {
    bishop_attacks_magic(square, occupied) | rook_attacks_magic(square, occupied)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_magic_initialization() {
        init_magics();
        // Test that tables are populated
        let attacks = unsafe { bishop_attacks_magic(Square::E4, Bitboard::EMPTY) };
        assert!(attacks.count() > 0);
    }

    #[test]
    fn test_bishop_magic_attacks() {
        init_magics();
        let attacks = unsafe { bishop_attacks_magic(Square::E4, Bitboard::EMPTY) };
        // Should attack all diagonals
        assert!(attacks.is_occupied(Square::D3));
        assert!(attacks.is_occupied(Square::F5));
    }

    #[test]
    fn test_rook_magic_attacks() {
        init_magics();
        let attacks = unsafe { rook_attacks_magic(Square::E4, Bitboard::EMPTY) };
        // Should attack entire rank and file
        assert!(attacks.is_occupied(Square::E1));
        assert!(attacks.is_occupied(Square::E8));
        assert!(attacks.is_occupied(Square::A4));
        assert!(attacks.is_occupied(Square::H4));
    }
}
