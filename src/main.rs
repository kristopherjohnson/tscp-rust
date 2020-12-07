// main.rs
// Tom Kerrigan's Simple Chess Program (TSCP)
//
// Copyright 1997 Tom Kerrigan
//
// Rust port by Kristopher Johnson

use std::io;
use std::io::prelude::*;

extern crate libc;

#[macro_use]
mod defs;

mod board;
mod book;
mod data;
mod eval;
mod scan;
mod search;
pub mod util;

use data::Data;
use defs::EMPTY;
use search::ThinkOutput::*;

const BANNER: [&str; 9] = [
    "",
    "Tom Kerrigan's Simple Chess Program (TSCP)",
    "version 1.81c, 2/3/19",
    "Copyright 2019 Tom Kerrigan",
    "",
    "(Rust port by Kristopher Johnson)",
    "",
    "\"help\" displays a list of commands.",
    "",
];

const HELP: [&str; 11] = [
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

fn print_lines(lines: &[&str]) {
    for line in lines.iter() {
        println!("{}", line);
    }
}

fn main() {
    print_lines(&BANNER);

    let mut d = Data::new();
    board::init_hash(&mut d);
    board::init_board(&mut d);
    book::open_book(&mut d);
    board::gen(&mut d);
    let mut computer_side = EMPTY;
    d.max_time = 1 << 25;
    d.max_depth = 4;
    loop {
        if d.side == computer_side {
            // computer's turn

            // think about the move and make it
            search::think(&mut d, NormalOutput);
            if d.pv[0][0].value() == 0 {
                println!("(no legal moves");
                computer_side = EMPTY;
                continue;
            }
            let m = d.pv[0][0].bytes();
            println!("Computer's move: {}", util::move_str(m));
            board::makemove(&mut d, m);
            d.ply = 0;
            board::gen(&mut d);
            util::print_result(&mut d);
            continue;
        }

        // get user input
        print!("tscp> ");
        io::stdout().flush().expect("unable to flush prompt output");
        let s = match scan::scan_token() {
            Ok(s) => s,
            Err(err) => {
                println!("input error: {}", err);
                return;
            }
        };
        if s.is_empty() {
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
                let n = match scan::scan_int() {
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
                let n = match scan::scan_int() {
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
                board::takeback(&mut d);
                d.ply = 0;
                board::gen(&mut d);
                continue;
            }
            "new" => {
                computer_side = EMPTY;
                board::init_board(&mut d);
                board::gen(&mut d);
                continue;
            }
            "d" => {
                util::print_board(&d);
                continue;
            }
            "bench" => {
                computer_side = EMPTY;
                util::bench(&mut d);
                continue;
            }
            "bye" => {
                println!("Share and enjoy!");
                break;
            }
            "xboard" => {
                util::xboard(&mut d);
                break;
            }
            "help" => {
                print_lines(&HELP);
                continue;
            }
            _ => {
                // maybe the user entered a move?
                let m = util::parse_move(&d, &s);
                if m == -1 {
                    println!("Illegal move.");
                } else {
                    let m = d.gen_dat[m as usize].m.bytes();
                    if !board::makemove(&mut d, m) {
                        println!("Illegal move.");
                    } else {
                        d.ply = 0;
                        board::gen(&mut d);
                        util::print_result(&mut d);
                    }
                }
            }
        }
    }
    book::close_book(&mut d);
}
