// book.rs
// Tom Kerrigan's Simple Chess Program (TSCP)
//
// Copyright 1997 Tom Kerrigan
//
// Rust port by Kristopher Johnson

use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

use super::util;

use super::data::Data;
use super::defs::Int;

// #rust The original C code keeps the book.txt file open throughout the
// lifetime of the program and re-reads its contents whenever it wants to look
// up a book move. In the Rust translation, we read the file's lines into
// Data.book_lines at initialization, close the file, and use that in-memory
// collection from then on.

/// open_book() opens the opening book file and initializes the random number
/// generator so we play random book moves.

pub fn open_book(d: &mut Data) {
    unsafe {
        // srand(time(NULL));
        libc::srand(libc::time(std::ptr::null_mut()) as u32);
    }

    let f = match File::open("book.txt") {
        Ok(file) => file,
        Err(err) => {
            println!("Opening book missing: {}.", err);
            d.book_lines = Vec::new();
            return;
        }
    };

    let reader = BufReader::new(f);
    let lines: Vec<String> = reader
        .lines()
        .map(|line| line.expect("unable to read line from book.txt"))
        .collect();
    d.book_lines = lines;
}

/// close_book() closes the book file. This is called when the program exits.

pub fn close_book(d: &mut Data) {
    d.book_lines = Vec::new();
}

/// book_move() returns a book move (in integer format) or -1 if there is no
/// book move.

pub fn book_move(d: &Data) -> Int {
    if d.hply > 25 {
        return -1;
    }

    // #rust In C, this variable is just "move", but that is a reserved word in
    // Rust.
    let mut move_: [Int; 50] = [0; 50]; // the possible book moves
    let mut count: [Int; 50] = [0; 50]; // number of occurrences of each move
    let mut moves = 0;
    let mut total_count = 0;

    // line is a string with the current line, e.g., "e2e4 e7e5 g1f3 "
    let mut line = String::from("");
    let mut j: Int;
    for i in 0..d.hply {
        line = line + &format!("{} ", util::move_str(d.hist_dat[i].m.bytes()));
    }

    // compare line to each line in the opening book
    for book_line in d.book_lines.iter() {
        // #rust The C code has a function book_match() to check whether the
        // prefix matches, but in Rust we can just call the standard library's
        // starts_with() method.
        if book_line.starts_with(&line) {
            // parse the book move that continues the line
            let m = util::parse_move(d, &book_line[line.len()..]);
            if m == -1 {
                continue;
            }
            let m = d.gen_dat[m as usize].m.value();

            // add the book move to the move list, or update the move's count
            j = 0;
            while j < moves {
                if move_[j as usize] == m {
                    count[j as usize] += 1;
                    break;
                }
                j += 1;
            }
            if j == moves {
                move_[moves as usize] = m;
                count[moves as usize] = 1;
                moves += 1;
            }
            total_count += 1;
        }
    }

    // no book moves?
    if moves == 0 {
        return -1;
    }

    // Think of total_count as the set of matching book lines. Randomly pick one
    // of those lines (j) and figure out which move j "corresponds" to.
    unsafe {
        j = (libc::rand() % (total_count as i32)) as Int;
    }
    for i in 0..(moves as usize) {
        j -= count[i];
        if j < 0 {
            return move_[i];
        }
    }

    -1
}
