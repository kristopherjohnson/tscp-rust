// board.rs
// Tom Kerrigan's Simple Chess Program (TSCP)
//
// Copyright 1997 Tom Kerrigan
//
// Rust port by Kristopher Johnson

/// xboard() is a substitute for main() that is XBoard and WinBoard compatible.
/// See the following page for details:
/// <http://www.research.digital.com/SRC/personal/mann/xboard/engine-intf.html>
use std::io;
use std::io::prelude::*;

use super::board;
use super::scan;
use super::search;
use super::util;

use super::data::Data;
use super::defs::{DARK, EMPTY, LIGHT};
use super::search::ThinkOutput::*;

pub fn xboard(d: &mut Data) {
    let mut post = NoOutput;

    unsafe {
        libc::signal(libc::SIGINT, libc::SIG_IGN);
    }
    println!();
    board::init_board(d);
    board::gen(d);
    let mut computer_side = EMPTY;
    loop {
        io::stdout()
            .flush()
            .expect("unable to flush standard output");
        if d.side == computer_side {
            search::think(d, post);
            if d.pv[0][0].value() == 0 {
                computer_side = EMPTY;
                continue;
            }
            let m = d.pv[0][0].bytes();
            println!("move {}", util::move_str(m));
            board::makemove(d, m);
            d.ply = 0;
            board::gen(d);
            util::print_result(d);
            continue;
        }
        let command = match scan::scan_token() {
            Ok(s) => s,
            Err(err) => {
                println!("input error: {}", err);
                return;
            }
        };
        if command.is_empty() {
            // #rust: EOF
            return;
        }
        match command.as_ref() {
            "xboard" => continue,
            "new" => {
                board::init_board(d);
                board::gen(d);
                computer_side = DARK;
            }
            "quit" => return,
            "force" => {
                computer_side = EMPTY;
            }
            "white" => {
                d.side = LIGHT;
                d.xside = DARK;
                board::gen(d);
                computer_side = DARK;
            }
            "black" => {
                d.side = DARK;
                d.xside = LIGHT;
                board::gen(d);
                computer_side = LIGHT;
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
            }
            "time" => {
                let n = match scan::scan_int() {
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
                search::think(d, NoOutput);
                if d.pv[0][0].value() == 0 {
                    continue;
                }
                println!("Hint: {}", util::move_str(d.pv[0][0].bytes()));
            }
            "undo" => {
                if d.hply == 0 {
                    continue;
                }
                board::takeback(d);
                d.ply = 0;
                board::gen(d);
            }
            "remove" => {
                if d.hply < 2 {
                    continue;
                }
                board::takeback(d);
                board::takeback(d);
                d.ply = 0;
                board::gen(d);
            }
            "post" => {
                post = XboardOutput;
            }
            "nopost" => {
                post = NoOutput;
            }
            _ => {
                let m = util::parse_move(d, &command);
                match m {
                    -1 => println!("Error (unknown command): {}", command),
                    _ => {
                        let m = d.gen_dat[m as usize].m.bytes();
                        if !board::makemove(d, m) {
                            println!("Error (unknown command): {}", command);
                        } else {
                            d.ply = 0;
                            board::gen(d);
                            util::print_result(d);
                        }
                    }
                }
            }
        }
    }
}
