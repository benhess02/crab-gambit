use std::fmt::Display;

#[derive(Clone, Copy, PartialEq)]
pub enum PieceType {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King
}

#[derive(Clone, Copy, PartialEq)]
pub struct Piece {
    pub is_white: bool,
    pub p_type: PieceType
}

impl Piece {
    pub fn white(p_type: PieceType) -> Piece {
        Piece { is_white: true, p_type }
    }

    pub fn black(p_type: PieceType) -> Piece {
        Piece { is_white: false, p_type }
    }
}

impl Display for Piece {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self.p_type {
            PieceType::Pawn => if self.is_white { "P" } else { "p" },
            PieceType::Rook => if self.is_white { "R" } else { "r" },
            PieceType::Bishop => if self.is_white { "B" } else { "b" },
            PieceType::Knight => if self.is_white { "N" } else { "n" },
            PieceType::Queen => if self.is_white { "Q" } else { "q" },
            PieceType::King => if self.is_white { "K" } else { "k" }
        })
    }
}