// main.rs
// Tom Kerrigan's Simple Chess Program (TSCP)
//
// Copyright 1997 Tom Kerrigan
//
// Rust port by Kristopher Johnson

// #rust Remove these when we finish translating all modules.
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_mut)]
#![allow(unused_macros)]

extern crate libc;

mod board;
mod data;
mod defs;

use crate::board::{gen, init_board, init_hash};
use crate::data::{MAX_DEPTH, MAX_TIME};
use crate::defs::EMPTY;

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
    // minimize the amount of unsafe code.
    unsafe {
        init_hash();
        init_board();
        // open_book();
        gen();
        let mut computer_side = EMPTY;
        MAX_TIME = 1 << 25;
        MAX_DEPTH = 4;
        loop {
            break;
        }
        // close_book()
    }
}
