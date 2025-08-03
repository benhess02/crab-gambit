use std::fmt::Display;

use crate::square::{ Square, RANK_NAMES, FILE_NAMES };

#[derive(Clone, Copy, PartialEq)]
pub struct Bitboard {
    bits: u64
}

impl Bitboard {
    pub fn empty() -> Bitboard {
        Bitboard { bits: 0 }
    }

    pub fn count(&self) -> u32 {
        self.bits.count_ones()
    }

    pub fn set(&mut self, square: Square, value: bool) {
        if square.is_valid() {
            let mask = 1 << (square.file * 8 + square.rank);
            if value {
                self.bits |= mask;
            } else {
                self.bits &= !mask;
            }
        }
    }

    pub fn get(&self, square: Square) -> bool {
        if square.is_valid() {
            self.bits & (1 << (square.file * 8 + square.rank)) != 0
        } else {
            false
        }
    }

    pub fn intersect(&self, other: Bitboard) -> Bitboard {
        Bitboard { bits: self.bits & other.bits }
    }

    pub fn union(&self, other: Bitboard) -> Bitboard {
        Bitboard { bits: self.bits | other.bits }
    }
}

impl Display for Bitboard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for rank in (0..8).rev() {
            write!(f, "{}  ", RANK_NAMES[rank as usize])?;
            for file in 0..8 {
                if self.get(Square { rank, file }) {
                    write!(f, " X")?;
                } else {
                    write!(f, " .")?;
                }
            }
            writeln!(f)?;
        }
        writeln!(f)?;
        write!(f, "   ")?;
        for file in 0..8 {
            write!(f, " {}", FILE_NAMES[file])?;
        }
        Ok(())
    }
}

impl From<Square> for Bitboard {
    fn from(square: Square) -> Self {
        let mut result = Bitboard::empty();
        result.set(square, true);
        return result;
    }
}

impl IntoIterator for Bitboard {
    type Item = Square;
    type IntoIter = BitboardIterator;

    fn into_iter(self) -> Self::IntoIter {
        BitboardIterator { bits: self.bits }
    }
}

pub struct BitboardIterator {
    bits: u64
}

impl Iterator for BitboardIterator {
    type Item = Square;

    fn next(&mut self) -> Option<Self::Item> {
        if self.bits == 0 {
            return None;
        }
        let index = self.bits.trailing_zeros();
        self.bits ^= 1 << index;
        return Some(Square {
            rank: (index % 8) as i8,
            file: (index / 8) as i8
        });
    }
}