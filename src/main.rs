// main.rs
// Tom Kerrigan's Simple Chess Program (TSCP)
//
// Copyright 1997 Tom Kerrigan
//
// Rust port by Kristopher Johnson

use std::char;
use std::io;
use std::io::prelude::*;
use std::time::{SystemTime, UNIX_EPOCH};

extern crate libc;

#[macro_use]
mod defs;

mod board;
mod book;
mod data;
mod eval;
mod scan;
mod search;

use crate::board::{
    gen, in_check, init_board, init_hash, makemove, set_hash, takeback,
};
use crate::book::{close_book, open_book};
use crate::data::{
    CASTLE, COLOR, EP, FIFTY, FIRST_MOVE, GEN_DAT, HPLY, MAX_DEPTH, MAX_TIME,
    NODES, PIECE, PIECE_CHAR, PLY, PV, SIDE, START_TIME, XSIDE,
};
use crate::defs::{Int, MoveBytes, BISHOP, DARK, EMPTY, KNIGHT, LIGHT, ROOK};
use crate::scan::{scan_int, scan_token};
use crate::search::{reps, think};

/// get_ms() returns the milliseconds elapsed since midnight, January 1, 1970

fn get_ms() -> u128 {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time error");
    duration.as_secs() as u128 * 1000 + duration.subsec_millis() as u128
}

fn main() {
    println!("");
    println!("Tom Kerrigan's Simple Chess Program (TSCP)");
    println!("version 1.81b, 3/10/16");
    println!("Copyright 2016 Tom Kerrigan");
    println!("");
    println!("(Rust port by Kristopher Johnson)");
    println!("");
    println!("\"help\" displays a list of commands.");
    println!("");

    // #rust TODO: Due to use of static mutable variables in the `data` module,
    // everything has to be marked "unsafe".  We know our usage is safe because
    // the program runs in a single thread, but we should eventually change the
    // members of the `data` module to be structs with associated methods and
    // minimize the amount of code considered unsafe in Rust.
    unsafe {
        init_hash();
        init_board();
        open_book();
        gen();
        let mut computer_side = EMPTY;
        MAX_TIME = 1 << 25;
        MAX_DEPTH = 4;
        loop {
            if SIDE == computer_side {
                // computer's turn

                // think about the move and make it
                think(1);
                if PV[0][0].u == 0 {
                    println!("(no legal moves");
                    computer_side = EMPTY;
                    continue;
                }
                println!("Computer's move: {}", move_str(&PV[0][0].b));
                makemove(&PV[0][0].b);
                PLY = 0;
                gen();
                print_result();
                continue;
            }

            // get user input
            print!("tscp> ");
            io::stdout().flush().expect("unable to flush prompt output");
            let s = match scan_token() {
                Ok(s) => s,
                Err(err) => {
                    println!("input error: {}", err);
                    return;
                }
            };
            if s.len() == 0 {
                // EOF
                return;
            }
            match s.as_ref() {
                "on" => {
                    computer_side = SIDE;
                    continue;
                }
                "off" => {
                    computer_side = EMPTY;
                    continue;
                }
                "st" => {
                    let n = match scan_int() {
                        Ok(n) => n,
                        Err(err) => {
                            println!("unable to read st argument: {}", err);
                            return;
                        }
                    };
                    MAX_TIME = n * 1000;
                    MAX_DEPTH = 32;
                    continue;
                }
                "sd" => {
                    let n = match scan_int() {
                        Ok(n) => n,
                        Err(err) => {
                            println!("unable to read sd argument: {}", err);
                            return;
                        }
                    };
                    MAX_DEPTH = n;
                    MAX_TIME = 1 << 25;
                    continue;
                }
                "undo" => {
                    if HPLY == 0 {
                        continue;
                    }
                    computer_side = EMPTY;
                    takeback();
                    PLY = 0;
                    gen();
                    continue;
                }
                "new" => {
                    computer_side = EMPTY;
                    init_board();
                    gen();
                    continue;
                }
                "d" => {
                    print_board();
                    continue;
                }
                "bench" => {
                    computer_side = EMPTY;
                    bench();
                    continue;
                }
                "bye" => {
                    println!("Share and enjoy!");
                    break;
                }
                "xboard" => {
                    xboard();
                    break;
                }
                "help" => {
                    println!("on - computer plays for the side to move");
                    println!("off - computer stops playing");
                    println!("st n - search for n seconds per move");
                    println!("sd n - search n ply per move");
                    println!("undo - takes back a move");
                    println!("new - starts a new game");
                    println!("d - display the board");
                    println!("bench - run the built-in benchmark");
                    println!("bye - exit the program");
                    println!("xboard - switch to XBoard mode");
                    println!(
                        "Enter moves in coordinate notation, e.g., e2e4, e7e8Q"
                    );
                    continue;
                }
                _ => {
                    // maybe the user entered a move?
                    let m = parse_move(&s);
                    if m == -1 || !makemove(&GEN_DAT[m as usize].m.b) {
                        println!("Illegal move.");
                    } else {
                        PLY = 0;
                        gen();
                        print_result();
                    }
                }
            }
        }
        close_book()
    }
}

/// parse the move s (in coordinate notation) and return the move's index in
/// GEN_DAT, or -1 if the move is illegal

unsafe fn parse_move(s: &str) -> Int {
    // convert string to vector of characters
    let s: Vec<char> = String::from(s).chars().collect();

    // make sure the string looks like a move
    let len = s.len();
    if len < 4
        || s[0] < 'a'
        || s[0] > 'h'
        || s[1] < '0'
        || s[1] > '9'
        || s[2] < 'a'
        || s[2] > 'h'
        || s[3] < '0'
        || s[3] > '9'
    {
        return -1;
    }

    let from = s[0] as u32 - 'a' as u32;
    let from = from + 8 * (8 - (s[1] as u32 - '0' as u32));
    let from = from as u8;
    let to = s[2] as u32 - 'a' as u32;
    let to = to + 8 * (8 - (s[3] as u32 - '0' as u32));
    let to = to as u8;

    for i in 0..FIRST_MOVE[1] {
        if GEN_DAT[i].m.b.from == from && GEN_DAT[i].m.b.to == to {
            // if the move is a promotion, handle the promotion piece; assume
            // that the promotion moves occur consecutively in GEN_DAT.
            if (GEN_DAT[i].m.b.bits & 32) != 0 {
                if s.len() < 5 {
                    return i as Int + 3; // assume it's a queen
                }
                match s[4] {
                    'N' | 'n' => return i as Int,
                    'B' | 'b' => return i as Int + 1,
                    'R' | 'r' => return i as Int + 2,
                    _ => return i as Int + 3, // assume it's a queen
                }
            }
            return i as Int;
        }
    }

    // didn't find the move
    -1
}

/// move_str returns a string with move m in coordinate notation

unsafe fn move_str(m: &MoveBytes) -> String {
    let from_col = char::from_u32_unchecked(col!(m.from) as u32 + 'a' as u32);
    let from_row = 8 - row!(m.from);
    let to_col = char::from_u32_unchecked(col!(m.to) as u32 + 'a' as u32);
    let to_row = 8 - row!(m.to);

    if (m.bits & 32) != 0 {
        let c = match m.promote as Int {
            KNIGHT => 'n',
            BISHOP => 'b',
            ROOK => 'r',
            _ => 'q',
        };
        format!("{}{}{}{}{}", from_col, from_row, to_col, to_row, c)
    } else {
        format!("{}{}{}{}", from_col, from_row, to_col, to_row)
    }
}

/// print_board() prints the board

unsafe fn print_board() {
    print!("\n8 ");
    for i in 0..64 {
        match COLOR[i] {
            EMPTY => {
                print!(" .");
            }
            LIGHT => {
                print!(" {}", PIECE_CHAR[PIECE[i as usize] as usize]);
            }
            DARK => {
                let light_char = PIECE_CHAR[PIECE[i as usize] as usize];
                let dark_char_u32 = light_char as u32 + 'a' as u32 - 'A' as u32;
                print!(" {}", char::from_u32_unchecked(dark_char_u32));
            }
            _ => {}
        }
        if (i + 1) % 8 == 0 && i != 63 {
            print!("\n{} ", 7 - row!(i));
        }
    }
    print!("\n\n   a b c d e f g h\n\n");
}

// xboard() is a substitute for main() that is XBoard and WinBoard compatible.
// See the following page for details:
// http://www.research.digital.com/SRC/personal/mann/xboard/engine-intf.html

unsafe fn xboard() {
    // #rust TODO
    println!("<xboard: unimplemented>");
}

/// print_result() checks to see if the game is over, and if so, prints the result.

unsafe fn print_result() {
    let mut i = 0;
    while i < FIRST_MOVE[1] {
        if makemove(&GEN_DAT[i].m.b) {
            takeback();
            break;
        }
        i += 1;
    }
    if i == FIRST_MOVE[1] {
        if in_check(SIDE) {
            if SIDE == LIGHT {
                println!("0-1 {{Black mates}}");
            } else {
                println!("1-0 {{White mates}}");
            }
        } else {
            println!("1/2-1/2 {{Stalemate}}");
        }
    } else if reps() == 2 {
        println!("1/2-1/2 {{Draw by repetition}}");
    } else if FIFTY >= 100 {
        println!("1/2-1/2 {{Draw by fifty move rule}}");
    }
}

// bench: This is a little benchmark code that calculates how many nodes per
// second TSCP searches.  It sets the position to move 17 of Bobby Fischer vs.
// J. Sherwin, New Jersey State Open Championship, 9/2/1957.  Then it searches
// five ply three times. It calculates nodes per second from the best time. */
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

unsafe fn bench() {
    let mut t: [Int; 3] = [0; 3];

    // setting the position to a non-initial position confuses the opening book
    // code.
    close_book();

    for i in 0..64 {
        COLOR[i] = BENCH_COLOR[i];
        PIECE[i] = BENCH_PIECE[i];
    }
    SIDE = LIGHT;
    XSIDE = DARK;
    CASTLE = 0;
    EP = -1;
    FIFTY = 0;
    PLY = 0;
    HPLY = 0;
    set_hash();
    print_board();
    MAX_TIME = 1 << 25;
    MAX_DEPTH = 5;
    for i in 0..3 {
        think(1);
        t[i] = (get_ms() - START_TIME) as Int;
        println!("Time: {} ms", t[i]);
    }
    if t[1] < t[0] {
        t[0] = t[1];
    }
    if t[2] < t[0] {
        t[0] = t[2];
    }
    println!("");
    println!("Nodes: {}", NODES);
    println!("Best time: {} ms", t[0]);
    if t[0] == 0 {
        println!("(invalid)");
        return;
    }
    let nps = (NODES as f64) / (t[0] as f64);
    let nps = nps * 1000.0;

    // Score: 1.00 = my Athlon XP 2000+
    println!(
        "Nodes per second: {} (Score: {:.3})",
        nps as i32,
        nps / 243169.0
    );

    init_board();
    open_book();
    gen();
}
