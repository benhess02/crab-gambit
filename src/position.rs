use std::fmt::Display;
use crate::bitboard::Bitboard;
use crate::square::{Square, RANK_NAMES, FILE_NAMES};
use crate::piece::{Piece, PieceType};
use crate::moves::{generate_moves, Move, PastMove};

#[derive(Clone, Copy, PartialEq)]
pub struct CastleState {
    pub can_short_castle: bool,
    pub can_long_castle: bool
}

#[derive(Clone, PartialEq)]
pub struct Position {
    pub white_to_play: bool,
    pub en_passant_target: Option<Square>,
    pub white_castle_state: CastleState,
    pub black_castle_state: CastleState,
    pub white_pieces: Bitboard,
    pub black_pieces: Bitboard,
    pub pawns: Bitboard,
    pub knights: Bitboard,
    pub bishops: Bitboard,
    pub rooks: Bitboard,
    pub queens: Bitboard,
    pub kings: Bitboard
}

impl Position {
    pub fn empty() -> Position {
        Position {
            white_to_play: true,
            en_passant_target: None,
            white_castle_state: CastleState { can_short_castle: true, can_long_castle: true },
            black_castle_state: CastleState { can_short_castle: true, can_long_castle: true },
            white_pieces: Bitboard::empty(),
            black_pieces: Bitboard::empty(),
            pawns: Bitboard::empty(),
            knights: Bitboard::empty(),
            bishops: Bitboard::empty(),
            rooks: Bitboard::empty(),
            queens: Bitboard::empty(),
            kings: Bitboard::empty()
        }
    }

    pub fn start() -> Position {
        const BACK_RANK: [PieceType; 8] = [
            PieceType::Rook,
            PieceType::Knight,
            PieceType::Bishop,
            PieceType::Queen,
            PieceType::King,
            PieceType::Bishop,
            PieceType::Knight,
            PieceType::Rook
        ];
        let mut pos = Self::empty();
        for file in 0..8 {
            pos.set_piece(Square { rank: 0, file }, Piece::white(BACK_RANK[file as usize]));
            pos.set_piece(Square { rank: 1, file }, Piece::white(PieceType::Pawn));
            pos.set_piece(Square { rank: 6, file }, Piece::black(PieceType::Pawn));
            pos.set_piece(Square { rank: 7, file }, Piece::black(BACK_RANK[file as usize]));
        }
        pos
    }

    pub fn remove_piece(&mut self, square: Square) {
        self.white_pieces.set(square, false);
        self.black_pieces.set(square, false);
        self.pawns.set(square, false);
        self.knights.set(square, false);
        self.bishops.set(square, false);
        self.rooks.set(square, false);
        self.queens.set(square, false);
        self.kings.set(square, false);
    }

    pub fn by_type_mut(&mut self, p_type: PieceType) -> &mut Bitboard {
        match p_type {
            PieceType::Pawn => &mut self.pawns,
            PieceType::Knight => &mut self.knights,
            PieceType::Bishop => &mut self.bishops,
            PieceType::Rook => &mut self.rooks,
            PieceType::Queen => &mut self.queens,
            PieceType::King => &mut self.kings
        }
    }

    pub fn by_color(&self, white: bool) -> Bitboard {
        if white {
            self.white_pieces
        } else {
            self.black_pieces
        }
    }

    pub fn by_color_mut(&mut self, white: bool) -> &mut Bitboard {
        if white {
            &mut self.white_pieces
        } else {
            &mut self.black_pieces
        }
    }

    pub fn all_pieces(&self) -> Bitboard {
        self.white_pieces.union(self.black_pieces)
    }

    pub fn get_piece_type(&self, square: Square) -> Option<PieceType> {
        if !self.all_pieces().get(square) {
            return None;
        }
        if self.pawns.get(square) { return Some(PieceType::Pawn); }
        if self.knights.get(square) { return Some(PieceType::Knight); }
        if self.bishops.get(square) { return Some(PieceType::Bishop); }
        if self.rooks.get(square) { return Some(PieceType::Rook); }
        if self.queens.get(square) { return Some(PieceType::Queen); }
        if self.kings.get(square) { return Some(PieceType::King); }
        return None;
    }

    pub fn get_peice(&self, square: Square) -> Option<Piece> {
        if let Some(p_type) = self.get_piece_type(square) {
            Some(Piece {
                is_white: self.white_pieces.get(square),
                p_type
            })
        } else {
            None
        }
    }

    pub fn set_or_remove_piece(&mut self, square: Square, piece: Option<Piece>) {
        self.remove_piece(square);
        if let Some(p) = piece {
            self.by_type_mut(p.p_type).set(square, true);
            self.by_color_mut(p.is_white).set(square, true);
        }
    }

    pub fn set_piece(&mut self, square: Square, piece: Piece) {
        self.set_or_remove_piece(square, Some(piece));
    }

    pub fn do_null_move(&mut self) {
        self.white_to_play = !self.white_to_play;
    }

    pub fn undo_null_move(&mut self) {
        self.white_to_play = !self.white_to_play;
    }

    pub fn do_move(&mut self, mv: Move) -> Result<PastMove, String> {
        if let Some(mut peice) = self.get_peice(mv.src) {
            let mut captured = self.get_peice(mv.dest);
            self.remove_piece(mv.src);

            // Promotion
            if let Some(promoted_type) = mv.promotion {
                peice.p_type = promoted_type;
            }

            self.set_piece(mv.dest, peice);

            // En passant
            if peice.p_type == PieceType::Pawn && let Some(target) = self.en_passant_target {
                if mv.src.rank == target.rank && mv.dest.file == target.file {
                    captured = self.get_peice(target);
                    self.remove_piece(target);
                }
            }
            
            // All information to create past move has been computed
            let result = PastMove {
                mv,
                captured_peice: captured,
                en_passant_target: self.en_passant_target
            };

            // En passant setup
            if peice.p_type == PieceType::Pawn && (mv.src.rank - mv.dest.rank).abs() == 2 {
                self.en_passant_target = Some(mv.dest);
            } else {
                self.en_passant_target = None;
            }

            // Castling
            let castle_state = if peice.is_white {
                &mut self.white_castle_state
            } else {
                &mut self.black_castle_state
            };
            if peice.p_type == PieceType::King {
                castle_state.can_short_castle = false;
                castle_state.can_long_castle = false;
            } else if peice.p_type == PieceType::Rook {
                if mv.src.file == 0 {
                    castle_state.can_long_castle = false;
                } else {
                    castle_state.can_short_castle = false;
                }
            }
            if peice.p_type == PieceType::King && (mv.src.file - mv.dest.file).abs() == 2 {
                if mv.dest.file > mv.src.file {
                    // Short castle
                    self.remove_piece(Square { rank: mv.src.rank, file: 7 });
                    self.set_piece(
                        Square { rank: mv.src.rank, file: 5 },
                        Piece { is_white: peice.is_white, p_type: PieceType::Rook }
                    );
                } else {
                    // Long castle
                    self.remove_piece(Square { rank: mv.src.rank, file: 0 });
                    self.set_piece(
                        Square { rank: mv.src.rank, file: 3 },
                        Piece { is_white: peice.is_white, p_type: PieceType::Rook }
                    );
                }
            }

            // Advance to next turn
            self.en_passant_target = None;
            self.white_to_play = !self.white_to_play;
            Ok(result)
        } else {
            Err(format!("Source square {} is empty", mv.src))
        }
    }

    pub fn undo_move(&mut self, past_move: PastMove) -> Result<(), String> {
        if let Some(mut peice) = self.get_peice(past_move.mv.dest) {
            if past_move.mv.promotion != None {
                peice.p_type = PieceType::Pawn;
            }
            self.set_piece(past_move.mv.src, peice);

            let mut captured_square = past_move.mv.dest;
            if peice.p_type == PieceType::Pawn && let Some(target) = past_move.en_passant_target {
                if past_move.mv.src.rank == target.rank && past_move.mv.dest.file == target.file {
                    self.remove_piece(past_move.mv.dest);
                    captured_square = target;
                }
            }

            // Castling
            if peice.p_type == PieceType::King && (past_move.mv.src.file - past_move.mv.dest.file).abs() == 2 {
                if past_move.mv.dest.file > past_move.mv.src.file {
                    // Short castle
                    self.remove_piece(Square { rank: past_move.mv.src.rank, file: 5 });
                    self.set_piece(
                        Square { rank: past_move.mv.src.rank, file: 7 },
                        Piece { is_white: peice.is_white, p_type: PieceType::Rook }
                    );
                } else {
                    // Long castle
                    self.remove_piece(Square { rank: past_move.mv.src.rank, file: 3 });
                    self.set_piece(
                        Square { rank: past_move.mv.src.rank, file: 0 },
                        Piece { is_white: peice.is_white, p_type: PieceType::Rook }
                    );
                }
            }

            self.set_or_remove_piece(captured_square, past_move.captured_peice);
            self.white_to_play = !self.white_to_play;
            Ok(())
        } else {
            Err(format!("Destination square {} is empty", past_move.mv.dest))
        }
    }

    pub fn is_check(&mut self) -> Result<bool, String> {
        let mut moves: Vec<Move> = Vec::new();
        self.do_null_move();
        generate_moves(&mut moves, self, true);
        for mv in moves {
            let past_move = self.do_move(mv)?;
            let kings = self.kings.count();
            self.undo_move(past_move)?;
            if kings < 2 {
                self.undo_null_move();
                return Ok(true);
            }
        }
        self.undo_null_move();
        return Ok(false);
    }
}

impl Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for rank in (0..8).rev() {
            write!(f, "{}  ", RANK_NAMES[rank as usize])?;
            for file in 0..8 {
                let peice = self.get_peice(Square { rank, file });
                if let Some(p) = peice {
                    write!(f, " {}", p)?;
                } else {
                    write!(f, "  ")?;
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