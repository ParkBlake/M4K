//! Core types for chess representation

use crate::asm::neon_ops::*;
use std::fmt;

/// A bitboard representing positions on the chess board
#[derive(Copy, Clone, PartialEq, Eq, Default, Hash)]
#[repr(transparent)]
pub struct Bitboard(pub u64);

impl Bitboard {
    /// Empty bitboard
    pub const EMPTY: Bitboard = Bitboard(0);

    /// Full bitboard (all squares)
    pub const ALL: Bitboard = Bitboard(0xFFFF_FFFF_FFFF_FFFF);

    /// Rank masks
    pub const RANK_1: Bitboard = Bitboard(0x0000_0000_0000_00FF);
    pub const RANK_2: Bitboard = Bitboard(0x0000_0000_0000_FF00);
    pub const RANK_3: Bitboard = Bitboard(0x0000_0000_00FF_0000);
    pub const RANK_4: Bitboard = Bitboard(0x0000_0000_FF00_0000);
    pub const RANK_5: Bitboard = Bitboard(0x0000_00FF_0000_0000);
    pub const RANK_6: Bitboard = Bitboard(0x0000_FF00_0000_0000);
    pub const RANK_7: Bitboard = Bitboard(0x00FF_0000_0000_0000);
    pub const RANK_8: Bitboard = Bitboard(0xFF00_0000_0000_0000);

    /// File masks
    pub const FILE_A: Bitboard = Bitboard(0x0101_0101_0101_0101);
    pub const FILE_B: Bitboard = Bitboard(0x0202_0202_0202_0202);
    pub const FILE_C: Bitboard = Bitboard(0x0404_0404_0404_0404);
    pub const FILE_D: Bitboard = Bitboard(0x0808_0808_0808_0808);
    pub const FILE_E: Bitboard = Bitboard(0x1010_1010_1010_1010);
    pub const FILE_F: Bitboard = Bitboard(0x2020_2020_2020_2020);
    pub const FILE_G: Bitboard = Bitboard(0x4040_4040_4040_4040);
    pub const FILE_H: Bitboard = Bitboard(0x8080_8080_8080_8080);

    /// Create a bitboard from a square
    #[inline(always)]
    pub const fn from_square(sq: Square) -> Self {
        Bitboard(1u64 << sq.0)
    }

    /// Check if a square is occupied
    #[inline(always)]
    pub const fn is_occupied(self, sq: Square) -> bool {
        (self.0 & (1u64 << sq.0)) != 0
    }

    /// Set a square
    #[inline(always)]
    pub fn set(&mut self, sq: Square) {
        self.0 |= 1u64 << sq.0;
    }

    /// Clear a square
    #[inline(always)]
    pub fn clear(&mut self, sq: Square) {
        self.0 &= !(1u64 << sq.0);
    }

    /// Toggle a square
    #[inline(always)]
    pub fn toggle(&mut self, sq: Square) {
        self.0 ^= 1u64 << sq.0;
    }

    /// Count the number of set bits using Neon
    #[inline(always)]
    pub fn count(self) -> u32 {
        popcnt(self.0)
    }

    /// Check if the bitboard is empty
    #[inline(always)]
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// Get the least significant bit
    #[inline(always)]
    pub fn lsb(self) -> Option<Square> {
        if self.0 == 0 {
            None
        } else {
            Some(Square(bitscan_forward(self.0) as u8))
        }
    }

    /// Pop the least significant bit and return it
    #[inline(always)]
    pub fn pop_lsb(&mut self) -> Option<Square> {
        let sq = self.lsb()?;
        self.0 = reset_lsb(self.0);
        Some(sq)
    }

    /// Iterator over set bits
    pub fn iter(self) -> BitboardIter {
        BitboardIter(self)
    }
}

/// Iterator over set bits in a bitboard
pub struct BitboardIter(Bitboard);

impl Iterator for BitboardIter {
    type Item = Square;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.pop_lsb()
    }
}

// Bitwise operations
impl std::ops::BitOr for Bitboard {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Bitboard(self.0 | rhs.0)
    }
}

impl std::ops::BitAnd for Bitboard {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self {
        Bitboard(self.0 & rhs.0)
    }
}

impl std::ops::BitXor for Bitboard {
    type Output = Self;
    fn bitxor(self, rhs: Self) -> Self {
        Bitboard(self.0 ^ rhs.0)
    }
}

impl std::ops::Not for Bitboard {
    type Output = Self;
    fn not(self) -> Self {
        Bitboard(!self.0)
    }
}

impl std::ops::BitOrAssign for Bitboard {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl std::ops::BitAndAssign for Bitboard {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0;
    }
}

impl std::ops::BitXorAssign for Bitboard {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0;
    }
}

impl fmt::Debug for Bitboard {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f)?;
        for rank in (0..8).rev() {
            for file in 0..8 {
                let sq = Square((rank * 8 + file) as u8);
                if self.is_occupied(sq) {
                    write!(f, "X ")?;
                } else {
                    write!(f, ". ")?;
                }
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

/// Represents a square on the chess board (0-63)
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct Square(pub u8);

impl Square {
    // Define all squares as constants
    pub const A1: Square = Square(0);
    pub const B1: Square = Square(1);
    pub const C1: Square = Square(2);
    pub const D1: Square = Square(3);
    pub const E1: Square = Square(4);
    pub const F1: Square = Square(5);
    pub const G1: Square = Square(6);
    pub const H1: Square = Square(7);

    pub const A2: Square = Square(8);
    pub const B2: Square = Square(9);
    pub const C2: Square = Square(10);
    pub const D2: Square = Square(11);
    pub const E2: Square = Square(12);
    pub const F2: Square = Square(13);
    pub const G2: Square = Square(14);
    pub const H2: Square = Square(15);

    pub const A3: Square = Square(16);
    pub const B3: Square = Square(17);
    pub const C3: Square = Square(18);
    pub const D3: Square = Square(19);
    pub const E3: Square = Square(20);
    pub const F3: Square = Square(21);
    pub const G3: Square = Square(22);
    pub const H3: Square = Square(23);

    pub const A4: Square = Square(24);
    pub const B4: Square = Square(25);
    pub const C4: Square = Square(26);
    pub const D4: Square = Square(27);
    pub const E4: Square = Square(28);
    pub const F4: Square = Square(29);
    pub const G4: Square = Square(30);
    pub const H4: Square = Square(31);

    pub const A5: Square = Square(32);
    pub const B5: Square = Square(33);
    pub const C5: Square = Square(34);
    pub const D5: Square = Square(35);
    pub const E5: Square = Square(36);
    pub const F5: Square = Square(37);
    pub const G5: Square = Square(38);
    pub const H5: Square = Square(39);

    pub const A6: Square = Square(40);
    pub const B6: Square = Square(41);
    pub const C6: Square = Square(42);
    pub const D6: Square = Square(43);
    pub const E6: Square = Square(44);
    pub const F6: Square = Square(45);
    pub const G6: Square = Square(46);
    pub const H6: Square = Square(47);

    pub const A7: Square = Square(48);
    pub const B7: Square = Square(49);
    pub const C7: Square = Square(50);
    pub const D7: Square = Square(51);
    pub const E7: Square = Square(52);
    pub const F7: Square = Square(53);
    pub const G7: Square = Square(54);
    pub const H7: Square = Square(55);

    pub const A8: Square = Square(56);
    pub const B8: Square = Square(57);
    pub const C8: Square = Square(58);
    pub const D8: Square = Square(59);
    pub const E8: Square = Square(60);
    pub const F8: Square = Square(61);
    pub const G8: Square = Square(62);
    pub const H8: Square = Square(63);

    /// Create a square from coordinates
    #[inline(always)]
    pub const fn new(file: u8, rank: u8) -> Self {
        Square(rank * 8 + file)
    }

    /// Get the file (0-7)
    #[inline(always)]
    pub const fn file(self) -> u8 {
        self.0 % 8
    }

    /// Get the rank (0-7)
    #[inline(always)]
    pub const fn rank(self) -> u8 {
        self.0 / 8
    }

    /// Convert to bitboard
    #[inline(always)]
    pub const fn to_bitboard(self) -> Bitboard {
        Bitboard::from_square(self)
    }
}

impl fmt::Debug for Square {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let file = (b'a' + self.file()) as char;
        let rank = (b'1' + self.rank()) as char;
        write!(f, "{}{}", file, rank)
    }
}

/// Piece type
#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
#[repr(u8)]
pub enum Piece {
    Pawn = 0,
    Knight = 1,
    Bishop = 2,
    Rook = 3,
    Queen = 4,
    King = 5,
}

impl Piece {
    pub fn from_u8(val: u8) -> Option<Piece> {
        match val {
            0 => Some(Piece::Pawn),
            1 => Some(Piece::Knight),
            2 => Some(Piece::Bishop),
            3 => Some(Piece::Rook),
            4 => Some(Piece::Queen),
            5 => Some(Piece::King),
            _ => None,
        }
    }
}

/// Color
#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
#[repr(u8)]
pub enum Color {
    White = 0,
    Black = 1,
}

impl Color {
    /// Get the opposite color
    #[inline(always)]
    pub const fn opposite(self) -> Self {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }

    pub fn from_u8(val: u8) -> Color {
        match val {
            0 => Color::White,
            1 => Color::Black,
            _ => Color::White,
        }
    }
}

/// Castle rights
#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
pub struct CastleRights(pub u8);

impl CastleRights {
    pub const NONE: CastleRights = CastleRights(0);
    pub const WHITE_KING: CastleRights = CastleRights(1);
    pub const WHITE_QUEEN: CastleRights = CastleRights(2);
    pub const BLACK_KING: CastleRights = CastleRights(4);
    pub const BLACK_QUEEN: CastleRights = CastleRights(8);
    pub const ALL: CastleRights = CastleRights(15);

    #[inline(always)]
    pub fn has(self, rights: CastleRights) -> bool {
        (self.0 & rights.0) != 0
    }

    #[inline(always)]
    pub fn add(&mut self, rights: CastleRights) {
        self.0 |= rights.0;
    }

    #[inline(always)]
    pub fn remove(&mut self, rights: CastleRights) {
        self.0 &= !rights.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bitboard_basics() {
        let mut bb = Bitboard::EMPTY;
        assert_eq!(bb.count(), 0);

        bb.set(Square::E4);
        assert!(bb.is_occupied(Square::E4));
        assert_eq!(bb.count(), 1);

        bb.set(Square::D4);
        assert_eq!(bb.count(), 2);

        bb.clear(Square::E4);
        assert!(!bb.is_occupied(Square::E4));
        assert_eq!(bb.count(), 1);
    }

    #[test]
    fn test_square_conversion() {
        let sq = Square::E4;
        assert_eq!(sq.file(), 4);
        assert_eq!(sq.rank(), 3);

        let sq2 = Square::new(4, 3);
        assert_eq!(sq, sq2);
    }
}
