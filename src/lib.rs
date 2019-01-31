// lib.rs
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
use crate::data::{Data, PIECE_CHAR};
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

/// tscp command loop.  Called by main().

pub fn tscp_main() {
    const BANNER: [&'static str; 9] = [
        "",
        "Tom Kerrigan's Simple Chess Program (TSCP)",
        "version 1.81b, 3/10/16",
        "Copyright 2016 Tom Kerrigan",
        "",
        "(Rust port by Kristopher Johnson)",
        "",
        "\"help\" displays a list of commands.",
        "",
    ];
    for line in BANNER.iter() {
        println!("{}", line);
    }

    let mut d = Data::new();

    // #rust TODO: Due to use of static mutable variables in the `data` module,
    // everything has to be marked "unsafe".  We know our usage is safe because
    // the program runs in a single thread, but we should eventually change the
    // members of the `data` module to be structs with associated methods and
    // minimize the amount of code considered unsafe in Rust.
    unsafe {
        init_hash(&mut d);
        init_board(&mut d);
        open_book();
        gen(&mut d);
        let mut computer_side = EMPTY;
        d.max_time = 1 << 25;
        d.max_depth = 4;
        loop {
            if d.side == computer_side {
                // computer's turn

                // think about the move and make it
                think(&mut d, 1);
                if d.pv[0][0].u == 0 {
                    println!("(no legal moves");
                    computer_side = EMPTY;
                    continue;
                }
                let m = d.pv[0][0].b;
                println!("Computer's move: {}", move_str(m));
                makemove(&mut d, m);
                d.ply = 0;
                gen(&mut d);
                print_result(&mut d);
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
                    computer_side = d.side;
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
                    d.max_time = n * 1000;
                    d.max_depth = 32;
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
                    d.max_depth = n;
                    d.max_time = 1 << 25;
                    continue;
                }
                "undo" => {
                    if d.hply == 0 {
                        continue;
                    }
                    computer_side = EMPTY;
                    takeback(&mut d);
                    d.ply = 0;
                    gen(&mut d);
                    continue;
                }
                "new" => {
                    computer_side = EMPTY;
                    init_board(&mut d);
                    gen(&mut d);
                    continue;
                }
                "d" => {
                    print_board(&d);
                    continue;
                }
                "bench" => {
                    computer_side = EMPTY;
                    bench(&mut d);
                    continue;
                }
                "bye" => {
                    println!("Share and enjoy!");
                    break;
                }
                "xboard" => {
                    xboard(&mut d);
                    break;
                }
                "help" => {
                    const HELP: [&'static str; 11] = [
                        "on - computer plays for the side to move",
                        "off - computer stops playing",
                        "st n - search for n seconds per move",
                        "sd n - search n ply per move",
                        "undo - takes back a move",
                        "new - starts a new game",
                        "d - display the board",
                        "bench - run the built-in benchmark",
                        "bye - exit the program",
                        "xboard - switch to XBoard mode",
                        "Enter moves in coordinate notation, e.g., e2e4, e7e8Q",
                    ];
                    for line in HELP.iter() {
                        println!("{}", line);
                    }
                }
                _ => {
                    // maybe the user entered a move?
                    let m = parse_move(&d, &s);
                    if m == -1 {
                        println!("Illegal move.");
                    } else {
                        let m = d.gen_dat[m as usize].m.b;
                        if !makemove(&mut d, m) {
                            println!("Illegal move.");
                        } else {
                            d.ply = 0;
                            gen(&mut d);
                            print_result(&mut d);
                        }
                    }
                }
            }
        }
        close_book()
    }
}

/// parse the move s (in coordinate notation) and return the move's index in
/// d.gen_dat, or -1 if the move is illegal

unsafe fn parse_move(d: &Data, s: &str) -> Int {
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

    for i in 0..d.first_move[1] {
        if d.gen_dat[i].m.b.from == from && d.gen_dat[i].m.b.to == to {
            // if the move is a promotion, handle the promotion piece; assume
            // that the promotion moves occur consecutively in d.gen_dat.
            if (d.gen_dat[i].m.b.bits & 32) != 0 {
                if s.len() < 5 {
                    return i as Int + 3; // assume it's a queen
                }
                return match s[4] {
                    'N' | 'n' => i,
                    'B' | 'b' => i + 1,
                    'R' | 'r' => i + 2,
                    _ => i + 3, // assume it's a queen
                } as Int;
            }
            return i as Int;
        }
    }

    // didn't find the move
    -1
}

/// move_str returns a string with move m in coordinate notation

unsafe fn move_str(m: MoveBytes) -> String {
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

unsafe fn print_board(d: &Data) {
    print!("\n8 ");
    for i in 0..64 {
        match d.color[i] {
            EMPTY => {
                print!(" .");
            }
            LIGHT => {
                print!(" {}", PIECE_CHAR[d.piece[i as usize] as usize]);
            }
            DARK => {
                let light_char = PIECE_CHAR[d.piece[i as usize] as usize];
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

unsafe fn xboard(d: &mut Data) {
    let mut post = 0;

    // #rust TODO: Find a way to do this in Rust:
    //signal(SIGINT, SIG_IGN);
    println!("");
    init_board(d);
    gen(d);
    let mut computer_side = EMPTY;
    loop {
        io::stdout()
            .flush()
            .expect("unable to flush standard output");
        if d.side == computer_side {
            think(d, post);
            if d.pv[0][0].u == 0 {
                computer_side = EMPTY;
                continue;
            }
            let m = d.pv[0][0].b;
            println!("move {}", move_str(m));
            makemove(d, m);
            d.ply = 0;
            gen(d);
            print_result(d);
            continue;
        }
        let command = match scan_token() {
            Ok(s) => s,
            Err(err) => {
                println!("input error: {}", err);
                return;
            }
        };
        if command.len() == 0 {
            // #rust: EOF
            return;
        }
        match command.as_ref() {
            "xboard" => continue,
            "new" => {
                init_board(d);
                gen(d);
                computer_side = DARK;
            }
            "quit" => return,
            "force" => {
                computer_side = EMPTY;
            }
            "white" => {
                d.side = LIGHT;
                d.xside = DARK;
                gen(d);
                computer_side = DARK;
            }
            "black" => {
                d.side = DARK;
                d.xside = LIGHT;
                gen(d);
                computer_side = LIGHT;
            }
            "st" => {
                let n = match scan_int() {
                    Ok(n) => n,
                    Err(err) => {
                        println!("unable to read st argument: {}", err);
                        return;
                    }
                };
                d.max_time = n * 1000;
                d.max_depth = 32;
            }
            "sd" => {
                let n = match scan_int() {
                    Ok(n) => n,
                    Err(err) => {
                        println!("unable to read sd argument: {}", err);
                        return;
                    }
                };
                d.max_depth = n;
                d.max_time = 1 << 25;
            }
            "time" => {
                let n = match scan_int() {
                    Ok(n) => n,
                    Err(err) => {
                        println!("unable to read time argument: {}", err);
                        return;
                    }
                };
                d.max_time = (n * 10) / 30;
                d.max_depth = 32;
            }
            "otim" => continue,
            "go" => {
                computer_side = d.side;
            }
            "hint" => {
                think(d, 0);
                if d.pv[0][0].u == 0 {
                    continue;
                }
                println!("Hint: {}", move_str(d.pv[0][0].b));
            }
            "undo" => {
                if d.hply == 0 {
                    continue;
                }
                takeback(d);
                d.ply = 0;
                gen(d);
            }
            "remove" => {
                if d.hply < 2 {
                    continue;
                }
                takeback(d);
                takeback(d);
                d.ply = 0;
                gen(d);
            }
            "post" => {
                post = 2;
            }
            "nopost" => {
                post = 0;
            }
            _ => {
                let m = parse_move(&d, &command);
                if m == -1 {
                    println!("Error (unknown command): {}", command);
                } else {
                    let m = d.gen_dat[m as usize].m.b;
                    if !makemove(d, m) {
                        println!("Error (unknown command): {}", command);
                    } else {
                        d.ply = 0;
                        gen(d);
                        print_result(d);
                    }
                }
            }
        }
    }
}

/// print_result() checks to see if the game is over, and if so, prints the result.

unsafe fn print_result(d: &mut Data) {
    let mut i = 0;
    while i < d.first_move[1] {
        if makemove(d, d.gen_dat[i].m.b) {
            takeback(d);
            break;
        }
        i += 1;
    }
    if i == d.first_move[1] {
        if in_check(&d, d.side) {
            if d.side == LIGHT {
                println!("0-1 {{Black mates}}");
            } else {
                println!("1-0 {{White mates}}");
            }
        } else {
            println!("1/2-1/2 {{Stalemate}}");
        }
    } else if reps(&d) == 2 {
        println!("1/2-1/2 {{Draw by repetition}}");
    } else if d.fifty >= 100 {
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

unsafe fn bench(d: &mut Data) {
    let mut t: [Int; 3] = [0; 3];

    // setting the position to a non-initial position confuses the opening book
    // code.
    close_book();

    for i in 0..64 {
        d.color[i] = BENCH_COLOR[i];
        d.piece[i] = BENCH_PIECE[i];
    }
    d.side = LIGHT;
    d.xside = DARK;
    d.castle = 0;
    d.ep = -1;
    d.fifty = 0;
    d.ply = 0;
    d.hply = 0;
    set_hash(d);
    print_board(d);
    d.max_time = 1 << 25;
    d.max_depth = 5;
    for i in 0..3 {
        think(d, 1);
        t[i] = (get_ms() - d.start_time) as Int;
        println!("Time: {} ms", t[i]);
    }
    if t[1] < t[0] {
        t[0] = t[1];
    }
    if t[2] < t[0] {
        t[0] = t[2];
    }
    println!("");
    println!("Nodes: {}", d.nodes);
    println!("Best time: {} ms", t[0]);
    if t[0] == 0 {
        println!("(invalid)");
        return;
    }
    let nps = (d.nodes as f64) / (t[0] as f64);
    let nps = nps * 1000.0;

    // Score: 1.00 = my Athlon XP 2000+
    println!(
        "Nodes per second: {} (Score: {:.3})",
        nps as i32,
        nps / 243169.0
    );

    init_board(d);
    open_book();
    gen(d);
}
