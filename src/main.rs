mod bitboard;
mod position;
mod square;
mod piece;
mod moves;
mod transposition;

use core::f32;
use std::fmt::Display;
use std::io;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use crate::bitboard::Bitboard;
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
    pub nodes: u32
}

impl SearchContext {
    fn new() -> Self {
        Self {
            move_lists: Vec::new(),
            nodes: 0
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

    if pos.kings.intersect(to_play).count() == 0 {
        return f32::NEG_INFINITY;
    }

    score += to_play.intersect(pos.kings).count() as f32 * 200f32;
    score += to_play.intersect(pos.queens).count() as f32 * 9f32;
    score += to_play.intersect(pos.rooks).count() as f32 * 5f32;
    score += to_play.intersect(pos.bishops).count() as f32 * 3f32;
    score += to_play.intersect(pos.knights).count() as f32 * 3f32;
    score += to_play.intersect(pos.pawns).count() as f32;

    let pawns = pos.pawns.intersect(to_play);
    for sq in pawns {
        // Doubled pawns
        if pawns.intersect(Bitboard::file(sq.file)).count() > 1 {
            score -= 0.25;
        }

        // Isolated pawn
        if pawns.intersect(Bitboard::file(sq.file + 1)).count() == 0
            && pawns.intersect(Bitboard::file(sq.file - 1)).count() == 0 {
            score -= 0.5;
        }
    }

    let mut moves: Vec<Move> = ctx.get_move_vec();
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

fn minimax(ctx: &mut SearchContext, pos: &mut Position, depth: i32, is_root: bool, is_done: &AtomicBool,
        mut alpha: f32, beta: f32) -> Result<(f32, Option<MoveChain>), String> {
    ctx.nodes += 1;

    if depth < 1 {
        let score = evaluate(ctx, pos);
        return Ok((score, None))
    }

    let mut moves: Vec<Move> = ctx.get_move_vec();
    if is_root {
        generate_legal_moves(&mut moves, pos)?;
    } else {
        generate_moves(&mut moves, pos, true);
        generate_moves(&mut moves, pos, false);
    }

    if moves.len() == 0 {
        if pos.is_check()? {
            return Ok((f32::NEG_INFINITY, None));
        } else {
            return Ok((0f32, None));
        }
    }

    let mut best_chain: Option<MoveChain> = None;
    for mv in &moves {
        let past_move = pos.do_move(mv.clone())?;
        let (mut score, chain) = minimax(ctx, pos, depth - 1, false, is_done, -beta, -alpha)?;
        score *= -1f32;
        pos.undo_move(past_move)?;

        if is_done.load(Ordering::Relaxed) {
            return Ok((f32::NEG_INFINITY, None));
        }

        if score > alpha {
            alpha = score;
            best_chain = Some(MoveChain::new(mv.clone(), chain));
            if alpha >= beta {
                break;
            }
        }
    }
    ctx.return_move_vec(moves);
    return Ok((alpha, best_chain));
}

fn iterative_deepening(ctx: Arc<Mutex<SearchContext>>, mut pos: Position, max_time: Duration) {

    let end_time = Instant::now() + max_time;

    let (tx, rx) = mpsc::channel::<Move>();
    let is_done = Arc::new(AtomicBool::new(false));
    let inner_is_done = is_done.clone();

    thread::spawn(move || {
        let mut _ctx = ctx.lock().unwrap();

        let mut depth: i32 = 1;

        loop {
            let start_time = Instant::now();
            _ctx.reset();
            let (score, best_mv) = minimax(
                &mut _ctx,
                &mut pos,
                depth,
                true,
                &inner_is_done,
                f32::NEG_INFINITY,
                f32::INFINITY
            ).unwrap();

            if inner_is_done.load(Ordering::Relaxed) {
                break;
            }

            let end_time = Instant::now();
            let minimax_time = end_time - start_time;
            let time_ms =  minimax_time.as_millis();
            let nps = (_ctx.nodes as f32 / minimax_time.as_secs_f32()) as u32;

            if let Some(mv) = best_mv {
                if let Err(_) = tx.send(mv.current) {
                    return;
                }
                println!("info depth {} time {} nodes {} nps {} score cp {} pv {}",
                    depth,
                    time_ms,
                    _ctx.nodes,
                    nps,
                    (score * 100f32) as i32,
                    mv
                );
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
    if let Some(mv) = best_move {
        is_done.store(true, Ordering::Relaxed);
        println!("bestmove {}", mv);
    } else {
        if let Ok(mv) = rx.recv() {
            println!("bestmove {}", mv);
        }
    }
}

fn main() -> Result<(), String> {
    let input = io::stdin();

    let mut line = String::new();

    let mut pos = Position::start();

    let ctx = Arc::new(Mutex::new(SearchContext::new()));

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
                iterative_deepening(ctx.clone(), pos.clone(), Duration::from_secs(6));
            }
            _ => {}
        }
    }
}
