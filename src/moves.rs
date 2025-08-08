use std::fmt::Display;
use std::str::FromStr;
use crate::position::Position;
use crate::square::Square;
use crate::piece::{Piece, PieceType};

#[derive(Clone, Copy, PartialEq)]
pub struct Move {
    pub src: Square,
    pub dest: Square,
    pub promotion: Option<PieceType>
}

impl Display for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.src, self.dest)?;
        if let Some(promotion_type) = self.promotion {
            write!(f, "{}", Piece::black(promotion_type))?;
        }
        Ok(())
    }
}

impl FromStr for Move {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Move {
            src: s[0..2].parse()?,
            dest: s[2..4].parse()?,
            promotion: match s.chars().nth(4) {
                Some('q') => Some(PieceType::Queen),
                Some('r') => Some(PieceType::Rook),
                Some('b') => Some(PieceType::Bishop),
                Some('n') => Some(PieceType::Knight),
                _ => None
            }
        })
    }
}

#[derive(Clone, Copy, PartialEq)]
pub struct PastMove {
    pub mv: Move,
    pub captured_peice: Option<Piece>,
    pub en_passant_target: Option<Square>
}

fn generate_move(moves: &mut Vec<Move>, pos: &Position, src: Square, dest: Square, capture: bool) -> bool {
    if !dest.is_valid() {
        return false;
    }
    if capture {
        let is_white = pos.white_pieces.get(src);
        if pos.by_color(!is_white).get(dest) {
            moves.push(Move {
                src,
                dest,
                promotion: None
            });
            return false;
        } else if pos.by_color(is_white).get(dest) {
            return false;
        }
    } else {
        if pos.all_pieces().get(dest) {
            return false;
        } else {
            moves.push(Move {
                src,
                dest,
                promotion: None
            });
        }
    }
    return true;
}

fn generate_direction_moves(moves: &mut Vec<Move>, pos: &Position, src: Square, dr: i8, df: i8, capture: bool) {
    let mut dest = src.clone();
    loop {
        dest.rank += dr;
        dest.file += df;
        if !generate_move(moves, pos, src, dest, capture) {
            return;
        }
    }
}

fn generate_pawn_moves(moves: &mut Vec<Move>, pos: &Position, src: Square, capture: bool) {
    let direction = if pos.white_pieces.get(src) { 1 } else { -1 };
    if capture {
        generate_move(moves, pos, src, src.add(direction, 1), true);
        generate_move(moves, pos, src, src.add(direction, -1), true);
        if let Some(target) = pos.en_passant_target {
            if target.rank == src.rank && (target.file - src.file).abs() == 1 {
                moves.push(Move {
                    src,
                    dest: target.add(direction, 0),
                    promotion: None
                });
            }
        }
    } else {
        let dest = src.add(direction, 0);
        if dest.rank == 7 || dest.rank == 0 {
            if !pos.all_pieces().get(dest) {
                moves.push(Move { src, dest, promotion: Some(PieceType::Queen) });
                moves.push(Move { src, dest, promotion: Some(PieceType::Rook) });
                moves.push(Move { src, dest, promotion: Some(PieceType::Bishop) });
                moves.push(Move { src, dest, promotion: Some(PieceType::Knight) });
            }
        } else {
            if generate_move(moves, pos, src, dest, false) {
                if src.rank == 1 || src.rank == 6 {
                    generate_move(moves, pos, src, dest.add(direction, 0), false);
                }
            }
        }
    }
}

fn generate_knight_moves(moves: &mut Vec<Move>, pos: &Position, src: Square, capture: bool) {
    generate_move(moves, pos, src, src.add(2, 1), capture);
    generate_move(moves, pos, src, src.add(2, -1), capture);

    generate_move(moves, pos, src, src.add(-2, 1), capture);
    generate_move(moves, pos, src, src.add(-2, -1), capture);

    generate_move(moves, pos, src, src.add(1, 2), capture);
    generate_move(moves, pos, src, src.add(-1, 2), capture);

    generate_move(moves, pos, src, src.add(1, -2), capture);
    generate_move(moves, pos, src, src.add(-1, -2), capture);
}

fn generate_rook_moves(moves: &mut Vec<Move>, pos: &Position, src: Square, capture: bool) {
    generate_direction_moves(moves, pos, src, 0, 1, capture);
    generate_direction_moves(moves, pos, src, 0, -1, capture);

    generate_direction_moves(moves, pos, src, 1, 0, capture);
    generate_direction_moves(moves, pos, src, -1, 0, capture);
}

fn generate_bishop_moves(moves: &mut Vec<Move>, pos: &Position, src: Square, capture: bool) {
    generate_direction_moves(moves, pos, src, 1, 1, capture);
    generate_direction_moves(moves, pos, src, -1, -1, capture);

    generate_direction_moves(moves, pos, src, -1, 1, capture);
    generate_direction_moves(moves, pos, src, 1, -1, capture);
}

fn generate_queen_moves(moves: &mut Vec<Move>, pos: &Position, src: Square, capture: bool) {
    generate_rook_moves(moves, pos, src, capture);
    generate_bishop_moves(moves, pos, src, capture);
}

fn generate_castle(moves: &mut Vec<Move>, pos: &Position, src: Square, dest: Square) {
    let castle_state = if pos.white_pieces.get(src) {
        &pos.white_castle_state
    } else {
        &pos.black_castle_state
    };

    let pieces = pos.all_pieces();
    if dest.file > src.file {
        if !castle_state.can_short_castle {
            return;
        }
        if pieces.get(src.add(0, 1)) {
            return;
        }
        if pieces.get(src.add(0, 2)) {
            return;
        }
    } else {
        if !castle_state.can_long_castle {
            return;
        }
        if pieces.get(src.add(0, -1)) {
            return;
        }
        if pieces.get(src.add(0, -2)) {
            return;
        }
        if pieces.get(src.add(0, -3)) {
            return;
        }
    }

    moves.push(Move {
        src,
        dest,
        promotion: None
    });
}

fn generate_king_moves(moves: &mut Vec<Move>, pos: &Position, src: Square, capture: bool) {
    generate_move(moves, pos, src, src.add(0, 1), capture);
    generate_move(moves, pos, src, src.add(0, -1), capture);

    generate_move(moves, pos, src, src.add(1, 0), capture);
    generate_move(moves, pos, src, src.add(-1, 0), capture);

    generate_move(moves, pos, src, src.add(1, 1), capture);
    generate_move(moves, pos, src, src.add(-1, -1), capture);

    generate_move(moves, pos, src, src.add(1, -1), capture);
    generate_move(moves, pos, src, src.add(-1, 1), capture);

    if !capture && src.rank == 0 && src.file == 4 {
        generate_castle(moves, pos, src, src.add(0, 2));
        generate_castle(moves, pos, src, src.add(0, -2));
    }
}

pub fn generate_moves(moves: &mut Vec<Move>, pos: &Position, capture: bool) {
    let to_play = pos.by_color(pos.white_to_play);

    if pos.kings.count() < 2 {
        return;
    }

    for square in pos.queens.intersect(to_play) {
        generate_queen_moves(moves, pos, square, capture);
    }
    for square in pos.knights.intersect(to_play) {
        generate_knight_moves(moves, pos, square, capture);
    }
    for square in pos.rooks.intersect(to_play) {
        generate_rook_moves(moves, pos, square, capture);
    }
    for square in pos.bishops.intersect(to_play) {
        generate_bishop_moves(moves, pos, square, capture);
    }
    for square in pos.pawns.intersect(to_play) {
        generate_pawn_moves(moves, pos, square, capture);
    }
    for square in pos.kings.intersect(to_play) {
        generate_king_moves(moves, pos, square, capture);
    }
}

pub fn generate_legal_moves(moves: &mut Vec<Move>, pos: &mut Position) -> Result<(), String> {
    let mut pseudo_legal: Vec<Move> = Vec::new();
    generate_moves(&mut pseudo_legal, pos, true);
    generate_moves(&mut pseudo_legal, pos, false);
    for mv in pseudo_legal {
        let past_move = pos.do_move(mv)?;
        pos.do_null_move();
        if !pos.is_check()? {
            moves.push(mv);
        }
        pos.undo_null_move();
        pos.undo_move(past_move)?;
    }
    Ok(())
}
