// main.rs
// Tom Kerrigan's Simple Chess Program (TSCP)
//
// Copyright 1997 Tom Kerrigan
//
// Rust port by Kristopher Johnson

use std::io;
use std::io::prelude::*;

use tscp::board::{gen, init_board, init_hash, makemove, takeback};
use tscp::book::{close_book, open_book};
use tscp::data::Data;
use tscp::defs::EMPTY;
use tscp::scan::{scan_int, scan_token};
use tscp::search::think;
use tscp::search::ThinkOutput::*;
use tscp::{bench, move_str, parse_move, print_board, print_result, xboard};

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

    let mut d = Data::new();
    init_hash(&mut d);
    init_board(&mut d);
    open_book(&mut d);
    gen(&mut d);
    let mut computer_side = EMPTY;
    d.max_time = 1 << 25;
    d.max_depth = 4;
    loop {
        if d.side == computer_side {
            // computer's turn

            // think about the move and make it
            think(&mut d, NormalOutput);
            if d.pv[0][0].value() == 0 {
                println!("(no legal moves");
                computer_side = EMPTY;
                continue;
            }
            let m = d.pv[0][0].bytes();
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
                let m = parse_move(&d, &s);
                if m == -1 {
                    println!("Illegal move.");
                } else {
                    let m = d.gen_dat[m as usize].m.bytes();
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
    close_book(&mut d);
}
