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

mod bench;
mod board;
mod book;
mod data;
mod eval;
mod scan;
mod search;
mod util;
mod xboard;

use data::Data;
use defs::EMPTY;
use search::ThinkOutput::*;

const BANNER: &str = "\n\
    Tom Kerrigan's Simple Chess Program (TSCP)\n\
    version 1.81c, 2/3/19\n\
    Copyright 2019 Tom Kerrigan\n\
    \n\
    (Rust port by Kristopher Johnson)\n\
    \n\
    \"help\" displays a list of commands.\n";

const HELP: &str = "on - computer plays for the side to move\n\
    off - computer stops playing\n\
    st n - search for n seconds per move\n\
    sd n - search n ply per move\n\
    undo - takes back a move\n\
    new - starts a new game\n\
    d - display the board\n\
    bench - run the built-in benchmark\n\
    bye - exit the program\n\
    xboard - switch to XBoard mode\n\
    Enter moves in coordinate notation, e.g., e2e4, e7e8Q";

fn main() {
    println!("{}", BANNER);

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
                bench::bench(&mut d);
                continue;
            }
            "bye" => {
                println!("Share and enjoy!");
                break;
            }
            "xboard" => {
                xboard::xboard(&mut d);
                break;
            }
            "help" => {
                println!("{}", HELP);
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
