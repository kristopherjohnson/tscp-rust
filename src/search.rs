// search.rs
// Tom Kerrigan's Simple Chess Program (TSCP)
//
// Copyright 1997 Tom Kerrigan
//
// Rust port by Kristopher Johnson

use crate::board::{gen, gen_caps, in_check, makemove, takeback};
use crate::book::book_move;
use crate::data::{
    FIFTY, FIRST_MOVE, FOLLOW_PV, GEN_DAT, HASH, HISTORY, HIST_DAT, HPLY,
    MAX_DEPTH, MAX_TIME, NODES, PLY, PV, PV_LENGTH, SIDE, START_TIME,
    STOP_TIME,
};
use crate::defs::{Int, Move, HIST_STACK, MAX_PLY};
use crate::eval::eval;
use crate::{get_ms, move_str};

use std::io::{stdout, Write};

static mut STOP_SEARCH: bool = false;

/// #rust The original C code uses setjmp/longjmp to unwind the stack and exit
/// if thinking-time expires during search().  Rust doesn't make it easy to use
/// setjmp/longjmp, so instead our search() will return Timeout in that case.

enum SearchResult {
    Value(Int),
    Timeout,
}

// think() calls search() iteratively. Search statistics are printed depending
// on the value of output:
// 0 = no output
// 1 = normal output
// 2 = xboard format output

pub unsafe fn think(output: Int) {
    // try the opening book first
    PV[0][0].u = book_move();
    if PV[0][0].u != -1 {
        return;
    }

    STOP_SEARCH = false;
    START_TIME = get_ms();
    STOP_TIME = START_TIME + MAX_TIME as u128;

    PLY = 0;
    NODES = 0;

    PV = [[Move { u: 0 }; MAX_PLY]; MAX_PLY];
    HISTORY = [[0; 64]; 64];
    if output == 1 {
        println!("ply      nodes  score  pv");
    }
    for i in 1..=MAX_DEPTH {
        FOLLOW_PV = true;
        match search(-10000, 10000, i) {
            SearchResult::Timeout => {
                // make sure to take back the line we were searching
                while PLY != 0 {
                    takeback();
                }
                return;
            }
            SearchResult::Value(x) => {
                if output == 1 {
                    print!("{:3}  {:9}  {:5} ", i, NODES, x);
                } else if output == 2 {
                    print!(
                        "{} {} {} {}",
                        i,
                        x,
                        (get_ms() - START_TIME) / 10,
                        NODES
                    );
                }
                if output != 0 {
                    for j in 0..PV_LENGTH[0] {
                        print!(" {}", move_str(&PV[0][j].b));
                    }
                    print!("\n");
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

unsafe fn search(alpha: Int, beta: Int, depth: Int) -> SearchResult {
    // we're as deep as we want to be; call quiesce() to get a reasonable score
    // and return it
    if depth == 0 {
        return quiesce(alpha, beta);
    }
    NODES += 1;

    // do some housekeeping every 1024 nodes
    if (NODES & 1023) == 0 {
        if !checkup() {
            return SearchResult::Timeout;
        }
    }

    PV_LENGTH[PLY] = PLY;

    // if this isn't the root of the search tree (where we have to pick a move
    // and can't simply return 0) then check to see if the position is a repeat.
    // if so, we can assume that this line is a draw and return 0.
    if PLY != 0 && reps() != 0 {
        return SearchResult::Value(0);
    }

    // are we too deep?
    if PLY >= MAX_PLY - 1 {
        return SearchResult::Value(eval());
    }
    if HPLY >= HIST_STACK - 1 {
        return SearchResult::Value(eval());
    }

    // are we in check? if so, we want to search deeper
    let mut depth = depth;
    let c = in_check(SIDE);
    if c {
        depth += 1;
    }
    gen();
    if FOLLOW_PV {
        // are we following the PV?
        sort_pv();
    }
    let mut f = false;
    let mut alpha = alpha;
    let mut x;

    // loop through the moves
    for i in FIRST_MOVE[PLY]..FIRST_MOVE[PLY + 1] {
        sort(i);
        if !makemove(&GEN_DAT[i].m.b) {
            continue;
        }
        f = true;
        match search(-beta, -alpha, depth - 1) {
            SearchResult::Timeout => {
                return SearchResult::Timeout;
            }
            SearchResult::Value(value) => {
                x = -value;
                takeback();
                if x > alpha {
                    // this move caused a cutoff, so increase the history value
                    // so it gets ordered high next time so we can search it
                    HISTORY[GEN_DAT[i].m.b.from as usize]
                        [GEN_DAT[i].m.b.to as usize] += depth;
                    if x >= beta {
                        return SearchResult::Value(beta);
                    }
                    alpha = x;

                    // update the PV
                    PV[PLY][PLY] = GEN_DAT[i].m;
                    for j in (PLY + 1)..PV_LENGTH[PLY + 1] {
                        PV[PLY][j] = PV[PLY + 1][j];
                    }
                    PV_LENGTH[PLY] = PV_LENGTH[PLY + 1];
                }
            }
        }
    }

    // no legal moves? then we're in checkmate or stalemate
    if !f {
        if c {
            return SearchResult::Value(-10000 + (PLY as Int));
        } else {
            return SearchResult::Value(0);
        }
    }

    if FIFTY >= 100 {
        return SearchResult::Value(0);
    }

    SearchResult::Value(alpha)
}

/// quiesce() is a recursive minimax search function with alpha-beta cutoffs. In
/// other words, negamax. It basically only searches capture sequences and
/// allows the evaluation function to cut the search off (and set alpha) The
/// idea is to find a position where there isn't a lot going on so the static
/// evaluation function will work.

unsafe fn quiesce(alpha: Int, beta: Int) -> SearchResult {
    NODES += 1;

    // do some housekeeping every 1024 nodes
    if (NODES & 1023) == 0 {
        if !checkup() {
            return SearchResult::Timeout;
        }
    }

    PV_LENGTH[PLY] = PLY;

    // are we too deep?
    if PLY >= MAX_PLY - 1 {
        return SearchResult::Value(eval());
    }
    if HPLY >= HIST_STACK - 1 {
        return SearchResult::Value(eval());
    }

    // check with the evaluation function
    let mut x = eval();
    if x >= beta {
        return SearchResult::Value(beta);
    }
    let mut alpha = alpha;
    if x > alpha {
        alpha = x;
    }

    gen_caps();
    if FOLLOW_PV {
        // are we following the PV?
        sort_pv();
    }

    // loop through the moves
    for i in FIRST_MOVE[PLY]..FIRST_MOVE[PLY + 1] {
        sort(i);
        if !makemove(&GEN_DAT[i].m.b) {
            continue;
        }
        match quiesce(-beta, -alpha) {
            SearchResult::Timeout => {
                return SearchResult::Timeout;
            }
            SearchResult::Value(value) => {
                x = -value;
                takeback();
                if x > alpha {
                    if x >= beta {
                        return SearchResult::Value(beta);
                    }
                    alpha = x;

                    // update the PV
                    PV[PLY][PLY] = GEN_DAT[i].m;
                    for j in (PLY + 1)..PV_LENGTH[PLY + 1] {
                        PV[PLY][j] = PV[PLY + 1][j];
                    }
                    PV_LENGTH[PLY] = PV_LENGTH[PLY + 1];
                }
            }
        }
    }
    SearchResult::Value(alpha)
}

/// reps() returns the number of times the current position has been repeated.
/// It compares the current value of hash to previous values.

pub unsafe fn reps() -> Int {
    let mut r = 0;
    for i in (HPLY - FIFTY as usize)..HPLY {
        if HIST_DAT[i].hash == HASH {
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

unsafe fn sort_pv() {
    FOLLOW_PV = false;
    for i in FIRST_MOVE[PLY]..FIRST_MOVE[PLY + 1] {
        if GEN_DAT[i].m.u == PV[0][PLY].u {
            FOLLOW_PV = true;
            GEN_DAT[i].score += 10000000;
            return;
        }
    }
}

/// sort() searches the current ply's move list from 'from' to the end to find
/// the move with the highest score. This it swaps that move and the 'from' move
/// so the move with the highest score gets searched next, and hopefully
/// produces a cutoff.

unsafe fn sort(from: usize) {
    let mut bs = -1; // best score
    let mut bi = from; // best i
    for i in from..FIRST_MOVE[PLY + 1] {
        if GEN_DAT[i].score > bs {
            bs = GEN_DAT[i].score;
            bi = i;
        }
    }
    std::mem::swap(&mut GEN_DAT[from], &mut GEN_DAT[bi]);
}

// checkup() is called once in a while during the search. If it returns false,
// the search time is up.

unsafe fn checkup() -> bool {
    // is the engine's time up? if so, unwind back to think()
    if get_ms() >= STOP_TIME {
        STOP_SEARCH = true;
        return false;
    }
    true
}
