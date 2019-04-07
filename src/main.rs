// main.rs
// Tom Kerrigan's Simple Chess Program (TSCP)
//
// Copyright 1997 Tom Kerrigan
//
// Rust port by Kristopher Johnson

use std::io;
use std::io::prelude::*;

use tscp::board::{gen, init_board, init_hash};
use tscp::data::Data;
use tscp::defs::EMPTY;
use tscp::engine::Engine;
use tscp::scan::{scan_int, scan_token};
use tscp::search::ThinkOutput::*;
use tscp::{bench, move_str, xboard};

fn main() {
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
    for line in BANNER.iter() {
        println!("{}", line);
    }

    let mut e = Engine::new();
    e.start();
    e.init_board();
    e.open_book();
    e.gen();
    e.set_max_time_and_depth(1 << 25, 4);

    let mut computer_side = EMPTY;
    loop {
        if e.side() == computer_side {
            // computer's turn

            // think about the move and make it
            let computer_move = e.think(NormalOutput);
            if computer_move.value() == 0 {
                println!("(no legal moves");
                computer_side = EMPTY;
                continue;
            }
            let m = computer_move.bytes();
            println!("Computer's move: {}", move_str(m));
            e.makemove(m);
            e.clear_ply();
            e.gen();
            e.print_result();
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
        if s.is_empty() {
            // EOF
            return;
        }
        match s.as_ref() {
            "on" => {
                computer_side = e.side();
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
                e.set_max_time_and_depth(n * 1000, 32);
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
                e.set_max_time_and_depth(1 << 25, n);
                continue;
            }
            "undo" => {
                if !e.can_takeback() {
                    continue;
                }
                computer_side = EMPTY;
                e.takeback();
                e.clear_ply();
                e.gen();
                continue;
            }
            "new" => {
                computer_side = EMPTY;
                e.init_board();
                e.gen();
                continue;
            }
            "d" => {
                e.print_board();
                continue;
            }
            "bench" => {
                computer_side = EMPTY;
                // #rust TODO: Support calling bench() with an Engine as
                // argument.  For now, just initialize a fresh Data instance and
                // use that.
                let mut d = Data::new();
                init_hash(&mut d);
                init_board(&mut d);
                gen(&mut d);
                bench(&mut d);
                continue;
            }
            "bye" => {
                println!("Share and enjoy!");
                break;
            }
            "xboard" => {
                xboard(&mut e);
                break;
            }
            "help" => {
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
                for line in HELP.iter() {
                    println!("{}", line);
                }
            }
            _ => {
                // maybe the user entered a move?
                match e.parse_move(s) {
                    None => {
                        println!("Illegal move.");
                    }
                    Some(m) => {
                        if !e.makemove(m) {
                            println!("Illegal move.");
                        } else {
                            e.clear_ply();
                            e.gen();
                            e.print_result();
                        }
                    }
                }
            }
        }
    }
    e.close_book();
    e.stop();
}
