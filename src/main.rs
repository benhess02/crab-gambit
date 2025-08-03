mod bitboard;
mod position;
mod square;
mod piece;
mod moves;

use core::f32;
use std::fmt::Display;
use std::io;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use crate::moves::{generate_legal_moves, generate_moves, Move};
use crate::position::Position;

struct MoveChain {
    current: Move,
    next: Option<Box<MoveChain>>
}

impl MoveChain {
    fn new(current: Move, next: Option<MoveChain>) -> Self {
        Self {
            current,
            next: match next {
                Some(c) => Some(Box::new(c)),
                None => None
            }
        }
    }
}

impl Display for MoveChain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.current)?;
        if let Some(next) = &self.next {
            write!(f, " {}", next)?;
        }
        Ok(())
    }
}

struct SearchContext {
    move_lists: Vec<Vec<Move>>,
    pub nodes: u32,
}

impl SearchContext {
    fn new() -> Self {
        Self {
            move_lists: Vec::new(),
            nodes: 0,
        }
    }

    fn reset(&mut self) {
        self.nodes = 0;
    }

    fn get_move_vec(&mut self) -> Vec<Move> {
        if let Some(mut move_vec) = self.move_lists.pop() {
            move_vec.clear();
            return move_vec;
        } else {
            return Vec::new();
        }
    }

    fn return_move_vec(&mut self, move_vec: Vec<Move>) {
        self.move_lists.push(move_vec);
    }
}

fn evaluate_to_play(ctx: &mut SearchContext, pos: &mut Position) -> f32 {
    let mut score = 0f32;
    let to_play = pos.by_color(pos.white_to_play);

    score += to_play.intersect(pos.kings).count() as f32 * 200f32;
    score += to_play.intersect(pos.queens).count() as f32 * 9f32;
    score += to_play.intersect(pos.rooks).count() as f32 * 5f32;
    score += to_play.intersect(pos.bishops).count() as f32 * 3f32;
    score += to_play.intersect(pos.knights).count() as f32 * 3f32;
    score += to_play.intersect(pos.pawns).count() as f32;

    let mut moves: Vec<Move> = ctx.get_move_vec();
    generate_moves(&mut moves, pos, true);
    generate_moves(&mut moves, pos, false);
    score += moves.len() as f32 * 0.1f32;
    ctx.return_move_vec(moves);

    return score;
}

fn evaluate(ctx: &mut SearchContext, pos: &mut Position) -> f32 {
    let mut score = evaluate_to_play(ctx, pos);
    pos.do_null_move();
    score -= evaluate_to_play(ctx, pos);
    pos.undo_null_move();
    return score;
}

fn minimax(ctx: &mut SearchContext, pos: &mut Position, depth: u32, final_layer: bool, mut score_min: f32, score_max: f32) -> Result<(f32, Option<MoveChain>), String> {
    ctx.nodes += 1;
    if depth < 1 {
        let score = evaluate(ctx, pos);
        return Ok((score, None));
    }

    let mut moves: Vec<Move> = ctx.get_move_vec();
    if final_layer {
        generate_legal_moves(&mut moves, pos)?;
    } else {
        generate_moves(&mut moves, pos, true);
        generate_moves(&mut moves, pos, false);
    }

    let mut best_score = f32::NEG_INFINITY;
    let mut best_chain: Option<MoveChain> = None;
    for mv in &moves {
        let past_move = pos.do_move(mv.clone())?;
        let (mut score, chain) = minimax(ctx, pos, depth - 1, false, -score_max, -score_min)?;
        score *= -1f32;
        pos.undo_move(past_move)?;

        if score > best_score {
            best_score = score;
            best_chain = Some(MoveChain::new(mv.clone(), chain));
            if best_score > score_max {
                break;
            }
            if best_score > score_min {
                score_min = best_score;
            }
        }
    }
    ctx.return_move_vec(moves);
    return Ok((best_score, best_chain));
}

fn iterative_deepening(mut pos: Position, max_time: Duration) {

    let end_time = Instant::now() + max_time;

    let (tx, rx) = mpsc::channel::<Move>();
    let is_done = Arc::new(Mutex::new(false));
    let inner_is_done = is_done.clone();

    thread::spawn(move || {
        let mut depth: u32 = 1;
        let mut ctx = SearchContext::new();

        while !*inner_is_done.lock().unwrap() {
            ctx.reset();
            let (score, best_mv) = minimax(
                &mut ctx,
                &mut pos,
                depth,
                true,
                f32::NEG_INFINITY,
                f32::INFINITY
            ).unwrap();

            if let Some(mv) = best_mv {
                if let Err(_) = tx.send(mv.current) {
                    return;
                }
                println!("info depth {} nodes {} score cp {} pv {}", depth, ctx.nodes, (score * 100f32) as i32, mv);
            }
            depth += 1;
        }
    });

    let mut best_move: Option<Move> = None;
    while Instant::now() < end_time {
        if let Ok(mv) = rx.recv_timeout(end_time - Instant::now()) {
            best_move = Some(mv);
        }
    }
    *is_done.lock().unwrap() = true;
    if let Some(mv) = best_move {
        println!("bestmove {}", mv);
    }
}

fn main() -> Result<(), String> {
    let input = io::stdin();

    let mut line = String::new();

    let mut pos = Position::start();

    loop {
        line.clear();
        input.read_line(&mut line).unwrap();
        let parts: Vec<&str> = line.split_whitespace().collect();

        match parts[0] {
            "uci" => {
                println!("id name CrabGambit");
                println!("id author Ben Hess");
                println!("uciok");
            },
            "isready" => {
                println!("readyok");
            },
            "quit" => {
                return Ok(());
            },
            "ucinewgame" => {
                pos = Position::start();
            },
            "position" => {
                pos = Position::start();
                if parts.len() > 3 {
                    for move_part in &parts[3..] {
                        pos.do_move(move_part.parse()?)?;
                    }
                }
            },
            "go" => {
                iterative_deepening(pos.clone(), Duration::from_secs(3));
            }
            _ => {}
        }
    }
}
