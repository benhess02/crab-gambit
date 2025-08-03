use std::{fmt::Display, str::FromStr};

pub const RANK_NAMES: [char; 8] = ['1', '2', '3', '4', '5', '6', '7', '8'];
pub const FILE_NAMES: [char; 8] = ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h'];

#[derive(Clone, Copy, PartialEq)]
pub struct Square {
    pub rank: i8,
    pub file: i8
}

impl Square {
    pub fn is_valid(&self) -> bool {
        self.file >= 0 && self.rank >= 0 && self.rank < 8 && self.file < 8
    }

    pub fn add(&self, ranks: i8, files: i8) -> Square {
        Square {
            rank: self.rank + ranks,
            file: self.file + files
        }
    }
}

impl FromStr for Square {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let file_char = s.chars().nth(0).unwrap();
        let rank_char = s.chars().nth(1).unwrap();
        Ok(Square {
            rank: RANK_NAMES.iter().position(|&c| c == rank_char).unwrap() as i8,
            file: FILE_NAMES.iter().position(|&c| c == file_char).unwrap() as i8
        })
    }
}

impl Display for Square {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", FILE_NAMES[self.file as usize], RANK_NAMES[self.rank as usize])
    }
}