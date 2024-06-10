use std::{
    collections::{HashMap, HashSet},
    fs,
    sync::{atomic::AtomicUsize, Arc, Condvar, Mutex},
    time::Instant,
};

use rayon::iter::{IntoParallelRefMutIterator, ParallelIterator};

use crate::{Board, GameBoard, State};

const MAX_SCORE: i32 = 10000;
const MIN_SCORE: i32 = -MAX_SCORE;
const WIN_SCORE: i32 = 1000;
const LOSS_SCORE: i32 = -WIN_SCORE;
const FAR_SCORE: i32 = 1;
const DRAW_SCORE: i32 = 0;

fn negamax(b: &mut Board, mut alpha: i32, mut beta: i32, depth: i32, nodes: &mut u64) -> i32 {
    *nodes += 1;
    match b.get_state() {
        1 => return WIN_SCORE + depth,
        2 => return LOSS_SCORE - depth,
        3 => return DRAW_SCORE,
        _ => (),
    }
    if depth == 0 {
        return FAR_SCORE;
    }
    let best = WIN_SCORE + depth - 1;
    if beta > best {
        beta = best;
        if alpha >= beta {
            return alpha;
        }
    }
    // Winning
    for to in 0..9 {
        if !b.is_winning_spot(to) {
            continue;
        }
        for size in 0..3 {
            if !b.is_free(size, to) {
                continue;
            }
            if b.is_left(size) {
                b.do_new_move(size, to);
                let mut score = negamax(b, -beta, -alpha, depth - 1, nodes);
                b.undo_new_move(size, to);

                if score != FAR_SCORE {
                    score = -score;
                }
                if score >= beta {
                    return score;
                }
                if score > alpha {
                    alpha = score;
                }
            }
            for from in 0..9 {
                if !b.is_movable(size, from) {
                    continue;
                }
                b.do_board_move(size, from, to);
                let mut score = negamax(b, -beta, -alpha, depth - 1, nodes);
                b.undo_board_move(size, from, to);

                if score != FAR_SCORE {
                    score = -score;
                }
                if score >= beta {
                    return score;
                }
                if score > alpha {
                    alpha = score;
                }
            }
        }
    }
    // New cover
    for size in (1..3).rev() {
        if !b.is_left(size) {
            continue;
        }
        for to in 0..9 {
            if !b.is_free(size, to) || !b.is_cover(size, to) {
                continue;
            }
            b.do_new_move(size, to);
            let mut score = negamax(b, -beta, -alpha, depth - 1, nodes);
            b.undo_new_move(size, to);

            if score != FAR_SCORE {
                score = -score;
            }
            if score >= beta {
                return score;
            }
            if score > alpha {
                alpha = score;
            }
        }
    }
    // New !cover
    for size in (0..3).rev() {
        if !b.is_left(size) {
            continue;
        }
        for to in 0..9 {
            if !b.is_free(size, to) || b.is_cover(size, to) {
                continue;
            }
            b.do_new_move(size, to);
            let mut score = negamax(b, -beta, -alpha, depth - 1, nodes);
            b.undo_new_move(size, to);

            if score != FAR_SCORE {
                score = -score;
            }
            if score >= beta {
                return score;
            }
            if score > alpha {
                alpha = score;
            }
        }
    }
    // New cover
    for size in (0..3).rev() {
        for to in 0..9 {
            if !b.is_free(size, to) {
                continue;
            }
            for from in 0..9 {
                if !b.is_movable(size, from) {
                    continue;
                }
                b.do_board_move(size, from, to);
                let mut score = negamax(b, -beta, -alpha, depth - 1, nodes);
                b.undo_board_move(size, from, to);

                if score != FAR_SCORE {
                    score = -score;
                }
                if score >= beta {
                    return score;
                }
                if score > alpha {
                    alpha = score;
                }
            }
        }
    }
    return alpha;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum EvalKind {
    TooFar,
    Loss,
    Draw,
    Win,
}

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Evaluation {
    pub kind: EvalKind,
    pub depth: u16,
    pub time: f32,
    pub nodes: u64,
}

fn deepening(b: &GameBoard, start_depth: i32, max_depth: i32) -> Evaluation {
    let mut nodes = 0;
    let mut depth = start_depth;
    let start = Instant::now();
    loop {
        let mut search_b = b.get_board().clone();
        let score = negamax(&mut search_b, MIN_SCORE, MAX_SCORE, depth, &mut nodes);
        if score != FAR_SCORE || depth >= max_depth {
            let time = start.elapsed().as_secs_f32();
            let mut eval = Evaluation {
                kind: EvalKind::TooFar,
                depth: depth as u16,
                time,
                nodes,
            };
            if score == FAR_SCORE {
                eval.kind = EvalKind::TooFar;
            } else if score == DRAW_SCORE {
                eval.kind = EvalKind::Draw;
            } else if score < 0 {
                eval.kind = EvalKind::Loss;
            } else if score > 0 {
                eval.kind = EvalKind::Win;
            }
            return eval;
        }
        depth += 1;
    }
}

#[derive(Clone)]
struct Store {
    cache: HashMap<u64, Evaluation>,
    eval: HashSet<u64>,
}

#[derive(Clone)]
pub struct Search {
    arc: Arc<(Mutex<Store>, Condvar)>,
}

impl Search {
    pub fn new() -> Search {
        let bytes = fs::read("scorebook").unwrap();
        let cache: HashMap<u64, Evaluation> = bincode::deserialize(&bytes).unwrap();
        println!("Search loaded: {} entries", cache.len());
        return Search {
            arc: Arc::new((
                Mutex::new(Store {
                    cache,
                    eval: HashSet::new(),
                }),
                Condvar::new(),
            )),
        };
    }

    pub fn evaluate(&mut self, b: &GameBoard, max_depth: i32) -> Evaluation {
        let id = b.get_max_id();
        let mut start_depth = std::cmp::min(10, max_depth);
        let mut guard = self.arc.0.lock().unwrap();
        if let Some(_) = guard.eval.get(&id) {
            loop {
                guard = self.arc.1.wait(guard).unwrap();
                if guard.eval.get(&id) == None {
                    break;
                }
            }
        }
        if let Some(e) = guard.cache.get(&id) {
            if e.kind != EvalKind::TooFar || e.depth >= max_depth as u16 {
                let mut eval = *e;
                eval.nodes = 0;
                eval.time = 0.0;
                return eval;
            }
            start_depth = std::cmp::max(start_depth, e.depth as i32);
        }
        guard.eval.insert(id);
        drop(guard);

        let eval = deepening(b, start_depth, max_depth);

        let mut guard = self.arc.0.lock().unwrap();
        guard.cache.insert(id, eval);
        guard.eval.remove(&id);
        self.arc.1.notify_all();
        drop(guard);

        return eval;
    }

    pub fn pre_evaluate(&mut self, depth: i32, max_depth: i32) {
        let mut board = GameBoard::new();
        let mut entries = Vec::new();
        let mut count = 0;
        let id = AtomicUsize::new(0);
        self.add_entries(&mut board, depth, max_depth, &mut count, &mut entries);
        println!("Count {}, added {}", count, entries.len());
        entries.par_iter_mut().for_each(|(s, b)| {
            let i = id.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            println!("eval {}: {}", i, b.get_max_id());
            let e = s.evaluate(b, max_depth);
            println!("done {}: {:?}", i, e);
        });
    }

    fn add_entries(
        &mut self,
        b: &mut GameBoard,
        depth: i32,
        max_depth: i32,
        count: &mut u64,
        entries: &mut Vec<(Search, GameBoard)>,
    ) {
        *count += 1;
        let guard = self.arc.0.lock().unwrap();
        'blk: {
            if let Some(e) = guard.cache.get(&b.get_max_id()) {
                if e.kind != EvalKind::TooFar || e.depth >= max_depth as u16 {
                    break 'blk;
                }
            }
            entries.push((self.clone(), b.clone()));
        }
        drop(guard);
        if *b.get_state() != State::InGame || depth == 0 {
            return;
        }
        for m in b.get_moves() {
            b.do_move(m);
            self.add_entries(b, depth - 1, max_depth, count, entries);
            b.undo_move();
        }
    }

    pub fn flush(&self) {
        let guard = self.arc.0.lock().unwrap();
        let bytes = bincode::serialize(&guard.cache).unwrap();
        fs::write("scorebook", bytes).unwrap();
        println!("Search saved: {} entries", guard.cache.len());
    }
}
