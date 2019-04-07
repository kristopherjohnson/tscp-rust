// search.rs
// Tom Kerrigan's Simple Chess Program (TSCP)
//
// Copyright 1997 Tom Kerrigan
//
// Rust port by Kristopher Johnson

use crate::board::{gen, gen_caps, in_check, makemove, takeback};
use crate::book::book_move;
use crate::data::Data;
use crate::defs::{Int, Move, HIST_STACK, MAX_PLY};
use crate::eval::eval;
use crate::{get_ms, move_str};

use std::io::{stdout, Write};

/// #rust The original C code uses setjmp/longjmp to unwind the stack and exit
/// if thinking-time expires during search().  Rust doesn't make it easy to use
/// setjmp/longjmp, so instead our search() will return Timeout in that case.

#[derive(Copy, Clone)]
enum SearchResult {
    Value(Int),
    Timeout,
}

/// output options for think()

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum ThinkOutput {
    NoOutput,
    NormalOutput,
    XboardOutput,
}

/// think() calls search() iteratively. Search statistics are printed depending
/// on the value of output.

pub fn think(d: &mut Data, output: ThinkOutput) {
    // try the opening book first
    d.pv[0][0].set_value(book_move(d));
    if d.pv[0][0].value() != -1 {
        return;
    }

    d.start_time = get_ms();
    d.stop_time = d.start_time + d.max_time as u128;

    d.ply = 0;
    d.nodes = 0;

    d.pv = [[Move { u: 0 }; MAX_PLY]; MAX_PLY];
    d.history = [[0; 64]; 64];
    if output == ThinkOutput::NormalOutput {
        println!("ply      nodes  score  pv");
    }
    for i in 1..=d.max_depth {
        d.follow_pv = true;
        match search(d, -10000, 10000, i) {
            SearchResult::Timeout => {
                // make sure to take back the line we were searching
                while d.ply != 0 {
                    takeback(d);
                }
                return;
            }
            SearchResult::Value(x) => {
                match output {
                    ThinkOutput::NoOutput => {}
                    ThinkOutput::NormalOutput => {
                        print!("{:3}  {:9}  {:5} ", i, d.nodes, x);
                    }
                    ThinkOutput::XboardOutput => {
                        print!(
                            "{} {} {} {}",
                            i,
                            x,
                            (get_ms() - d.start_time) / 10,
                            d.nodes
                        );
                    }
                }
                if output != ThinkOutput::NoOutput {
                    for j in 0..d.pv_length[0] {
                        print!(" {}", move_str(d.pv[0][j].bytes()));
                    }
                    println!();
                    stdout().flush().expect("flush");
                }
                if x > 9000 || x < -9000 {
                    return;
                }
            }
        }
    }
}

/// search() does just that, in negamax fashion

#[allow(clippy::manual_memcpy)]
fn search(d: &mut Data, alpha: Int, beta: Int, depth: Int) -> SearchResult {
    // we're as deep as we want to be; call quiesce() to get a reasonable score
    // and return it
    if depth == 0 {
        return quiesce(d, alpha, beta);
    }
    d.nodes += 1;

    // do some housekeeping every 1024 nodes
    if (d.nodes & 1023) == 0 && !checkup(d) {
        return SearchResult::Timeout;
    }

    d.pv_length[d.ply] = d.ply;

    // if this isn't the root of the search tree (where we have to pick a move
    // and can't simply return 0) then check to see if the position is a repeat.
    // if so, we can assume that this line is a draw and return 0.
    if d.ply != 0 && reps(d) != 0 {
        return SearchResult::Value(0);
    }

    // are we too deep?
    if d.ply >= MAX_PLY - 1 {
        return SearchResult::Value(eval(d));
    }
    if d.hply >= HIST_STACK - 1 {
        return SearchResult::Value(eval(d));
    }

    // are we in check? if so, we want to search deeper
    let mut depth = depth;
    let c = in_check(d, d.side);
    if c {
        depth += 1;
    }
    gen(d);
    if d.follow_pv {
        // are we following the PV?
        sort_pv(d);
    }
    let mut f = false;
    let mut alpha = alpha;
    let mut x;

    // loop through the moves
    for i in d.first_move[d.ply]..d.first_move[d.ply + 1] {
        sort(d, i);
        if !makemove(d, d.gen_dat[i].m.bytes()) {
            continue;
        }
        f = true;
        match search(d, -beta, -alpha, depth - 1) {
            SearchResult::Timeout => {
                return SearchResult::Timeout;
            }
            SearchResult::Value(value) => {
                x = -value;
                takeback(d);
                if x > alpha {
                    // this move caused a cutoff, so increase the history value
                    // so it gets ordered high next time so we can search it
                    d.history[d.gen_dat[i].m.bytes().from as usize]
                        [d.gen_dat[i].m.bytes().to as usize] += depth;
                    if x >= beta {
                        return SearchResult::Value(beta);
                    }
                    alpha = x;

                    // update the PV
                    d.pv[d.ply][d.ply] = d.gen_dat[i].m;
                    // #rust TODO: use split_at_mut/clone_from_slice instead of
                    // manual element-by-element copy here.  (And remove the
                    // #[allow(clippy::manual_memcpy)] annotation.)
                    for j in (d.ply + 1)..d.pv_length[d.ply + 1] {
                        d.pv[d.ply][j] = d.pv[d.ply + 1][j];
                    }
                    d.pv_length[d.ply] = d.pv_length[d.ply + 1];
                }
            }
        }
    }

    // no legal moves? then we're in checkmate or stalemate
    if !f {
        if c {
            return SearchResult::Value(-10000 + (d.ply as Int));
        } else {
            return SearchResult::Value(0);
        }
    }

    if d.fifty >= 100 {
        return SearchResult::Value(0);
    }

    SearchResult::Value(alpha)
}

/// quiesce() is a recursive minimax search function with alpha-beta cutoffs. In
/// other words, negamax. It basically only searches capture sequences and
/// allows the evaluation function to cut the search off (and set alpha) The
/// idea is to find a position where there isn't a lot going on so the static
/// evaluation function will work.

#[allow(clippy::manual_memcpy)]
fn quiesce(d: &mut Data, alpha: Int, beta: Int) -> SearchResult {
    d.nodes += 1;

    // do some housekeeping every 1024 nodes
    if (d.nodes & 1023) == 0 && !checkup(d) {
        return SearchResult::Timeout;
    }

    d.pv_length[d.ply] = d.ply;

    // are we too deep?
    if d.ply >= MAX_PLY - 1 {
        return SearchResult::Value(eval(d));
    }
    if d.hply >= HIST_STACK - 1 {
        return SearchResult::Value(eval(d));
    }

    // check with the evaluation function
    let mut x = eval(d);
    if x >= beta {
        return SearchResult::Value(beta);
    }
    let mut alpha = alpha;
    if x > alpha {
        alpha = x;
    }

    gen_caps(d);
    if d.follow_pv {
        // are we following the PV?
        sort_pv(d);
    }

    // loop through the moves
    for i in d.first_move[d.ply]..d.first_move[d.ply + 1] {
        sort(d, i);
        if !makemove(d, d.gen_dat[i].m.bytes()) {
            continue;
        }
        match quiesce(d, -beta, -alpha) {
            SearchResult::Timeout => {
                return SearchResult::Timeout;
            }
            SearchResult::Value(value) => {
                x = -value;
                takeback(d);
                if x > alpha {
                    if x >= beta {
                        return SearchResult::Value(beta);
                    }
                    alpha = x;

                    // update the PV
                    d.pv[d.ply][d.ply] = d.gen_dat[i].m;
                    // #rust TODO: use split_at_mut/clone_from_slice instead of
                    // manual element-by-element copy here.  (And remove the
                    // #[allow(clippy::manual_memcpy)] annotation.)
                    for j in (d.ply + 1)..d.pv_length[d.ply + 1] {
                        d.pv[d.ply][j] = d.pv[d.ply + 1][j];
                    }
                    d.pv_length[d.ply] = d.pv_length[d.ply + 1];
                }
            }
        }
    }
    SearchResult::Value(alpha)
}

/// reps() returns the number of times the current position has been repeated.
/// It compares the current value of hash to previous values.

pub fn reps(d: &Data) -> Int {
    let mut r = 0;
    for i in (d.hply - d.fifty as usize)..d.hply {
        if d.hist_dat[i].hash == d.hash {
            r += 1;
        }
    }
    r
}

/// sort_pv() is called when the search function is following the PV (Principal
/// Variation). It looks through the current ply's move list to see if the PV
/// move is there. If so, it adds 10,000,000 to the move's score so it's played
/// first by the search function. If not, follow_pv remains false and search()
/// stops calling sort_pv().

fn sort_pv(d: &mut Data) {
    d.follow_pv = false;
    for i in d.first_move[d.ply]..d.first_move[d.ply + 1] {
        if d.gen_dat[i].m.value() == d.pv[0][d.ply].value() {
            d.follow_pv = true;
            d.gen_dat[i].score += 10_000_000;
            return;
        }
    }
}

/// sort() searches the current ply's move list from 'from' to the end to find
/// the move with the highest score. This it swaps that move and the 'from' move
/// so the move with the highest score gets searched next, and hopefully
/// produces a cutoff.

fn sort(d: &mut Data, from: usize) {
    let mut bs = -1; // best score
    let mut bi = from; // best i
    for i in from..d.first_move[d.ply + 1] {
        if d.gen_dat[i].score > bs {
            bs = d.gen_dat[i].score;
            bi = i;
        }
    }
    d.gen_dat.swap(from, bi);
}

// checkup() is called once in a while during the search. If it returns false,
// the search time is up.

fn checkup(d: &Data) -> bool {
    // is the engine's time up? if so, unwind back to think()
    if get_ms() >= d.stop_time {
        return false;
    }
    true
}
