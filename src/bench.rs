// board.rs
// Tom Kerrigan's Simple Chess Program (TSCP)
//
// Copyright 1997 Tom Kerrigan
//
// Rust port by Kristopher Johnson

use super::board;
use super::book;
use super::search;
use super::util;

use super::data::Data;
use super::defs::{Int, DARK, LIGHT};
use super::search::ThinkOutput::*;

#[rustfmt::skip]
const BENCH_COLOR: [Int; 64] = [
    6, 1, 1, 6, 6, 1, 1, 6,
    1, 6, 6, 6, 6, 1, 1, 1,
    6, 1, 6, 1, 1, 6, 1, 6,
    6, 6, 6, 1, 6, 6, 0, 6,
    6, 6, 1, 0, 6, 6, 6, 6,
    6, 6, 0, 6, 6, 6, 0, 6,
    0, 0, 0, 6, 6, 0, 0, 0,
    0, 6, 0, 6, 0, 6, 0, 6
];

#[rustfmt::skip]
const BENCH_PIECE: [Int; 64] = [
    6, 3, 2, 6, 6, 3, 5, 6,
    0, 6, 6, 6, 6, 0, 0, 0,
    6, 0, 6, 4, 0, 6, 1, 6,
    6, 6, 6, 1, 6, 6, 1, 6,
    6, 6, 0, 0, 6, 6, 6, 6,
    6, 6, 0, 6, 6, 6, 0, 6,
    0, 0, 4, 6, 6, 0, 2, 0,
    3, 6, 2, 6, 3, 6, 5, 6
];

/// bench: This is a little benchmark code that calculates how many nodes per
/// second TSCP searches.  It sets the position to move 17 of Bobby Fischer vs.
/// J. Sherwin, New Jersey State Open Championship, 9/2/1957.  Then it searches
/// five ply three times. It calculates nodes per second from the best time.

pub fn bench(d: &mut Data) {
    // setting the position to a non-initial position confuses the opening book
    // code.
    book::close_book(d);

    d.color[..].clone_from_slice(&BENCH_COLOR[..]);
    d.piece[..].clone_from_slice(&BENCH_PIECE[..]);
    d.side = LIGHT;
    d.xside = DARK;
    d.castle = 0;
    d.ep = -1;
    d.fifty = 0;
    d.ply = 0;
    d.hply = 0;
    board::set_hash(d);
    util::print_board(d);
    d.max_time = 1 << 25;
    d.max_depth = 5;

    let mut t: [Int; 3] = [0; 3];
    for x in &mut t {
        search::think(d, NormalOutput);
        *x = (util::get_ms() - d.start_time) as Int;
        println!("Time: {} ms", *x);
    }
    t.sort_unstable();

    println!();
    println!("Nodes: {}", d.nodes);
    println!("Best time: {} ms", t[0]);
    if t[0] == 0 {
        println!("(invalid)");
        return;
    }
    let nps = d.nodes / t[0];
    let nps = nps as f64 * 1000.0;

    // Score: 1.00 = my Athlon XP 2000+
    println!(
        "Nodes per second: {} (Score: {:.3})",
        nps as i32,
        nps / 243_169.0
    );

    board::init_board(d);
    book::open_book(d);
    board::gen(d);
}

#[cfg(test)]
mod tests {
    use super::*;

    use super::super::board;
    use super::super::book;
    use super::super::search;
    use super::super::util;

    use super::super::data::Data;
    use super::super::defs::{Int, DARK, LIGHT};

    /// This code is the same as bench::bench(), except that it only performs
    /// one iteration and checks the results rather than printing them.
    ///
    /// It is ignored by default because it takes a pretty long time to run in a
    /// debug build.
    #[test]
    #[ignore]
    fn test_bench() {
        let mut d = Data::new();
        board::init_hash(&mut d);
        board::init_board(&mut d);
        book::open_book(&mut d);
        board::gen(&mut d);

        // TODO: factor out this initialization code for use by both bench() and
        // test_bench().
        book::close_book(&mut d);
        d.color[..].clone_from_slice(&BENCH_COLOR[..]);
        d.piece[..].clone_from_slice(&BENCH_PIECE[..]);
        d.side = LIGHT;
        d.xside = DARK;
        d.castle = 0;
        d.ep = -1;
        d.fifty = 0;
        d.ply = 0;
        d.hply = 0;
        board::set_hash(&mut d);
        d.max_time = 1 << 25;
        d.max_depth = 5;

        search::think(&mut d, NormalOutput);
        let _ = (util::get_ms() - d.start_time) as Int;

        // TODO: Verify these expected results (from C tscp on macOS)
        //
        // Note: Current tscp-rust gets different bench results.  Results
        // matched until the 05ce54c commit on 2020-12-02, so we need to figure
        // out what went wrong there and since then.
        //
        // ply      nodes  score  pv
        //  1        130     20  c1e3
        //  2       3441      5  g5e4 d6c7
        //  3       8911     30  g5e4 d6c7 c1e3
        //  4     141367     10  g5e4 d6c7 c1e3 c8d7
        //  5     550778     26  c2a4 d6c7 g2d5 e6d5 c1e3

        // TODO: measure performance
    }
}
