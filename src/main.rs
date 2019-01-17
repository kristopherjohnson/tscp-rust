//  main.rs
//  Tom Kerrigan's Simple Chess Program (TSCP)
//
//  Copyright 1997 Tom Kerrigan
//
//  Rust port by Kristopher Johnson

mod board;
mod data;
mod defs;

use crate::board::init_board;

fn main() {
    println!("");
    println!("Tom Kerrigan's Simple Chess Program (TSCP)");
    println!("version 1.81b, 3/10/16");
    println!("Copyright 2016 Tom Kerrigan");
    println!("");
    println!("Rust port by Kristopher Johnson");
    println!("");
    println!("\"help\" displays a list of commands.");
    println!("");

    // #rust TODO: Do to use of static mutable variables in the data module,
    // everything has to be marked "unsafe".  Our usage is safe because the
    // program runs in a single thread, but we should eventually change the
    // definitions in the data module to be structs with associated methods and
    // minimize the amount of unsafe code.
    unsafe {
        init_board();
    }
}
