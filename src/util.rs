// lib.rs
// Tom Kerrigan's Simple Chess Program (TSCP)
//
// Copyright 1997 Tom Kerrigan
//
// Rust port by Kristopher Johnson

use std::time::{SystemTime, UNIX_EPOCH};

use super::board;
use super::search;

use super::data::{Data, PIECE_CHAR};
use super::defs::{Int, MoveBytes, BISHOP, DARK, EMPTY, KNIGHT, LIGHT, ROOK};

/// get_ms() returns the milliseconds elapsed since midnight, January 1, 1970

pub fn get_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time error")
        .as_millis()
}

/// parse the move s (in coordinate notation) and return the move's index in
/// d.gen_dat, or -1 if the move is illegal

pub fn parse_move(d: &Data, s: &str) -> Int {
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
        if d.gen_dat[i].m.bytes().from == from && d.gen_dat[i].m.bytes().to == to {
            // if the move is a promotion, handle the promotion piece; assume
            // that the promotion moves occur consecutively in d.gen_dat.
            if (d.gen_dat[i].m.bytes().bits & 32) != 0 {
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

pub fn move_str(m: MoveBytes) -> String {
    unsafe {
        let from_col = std::char::from_u32_unchecked(col!(m.from) as u32 + 'a' as u32);
        let from_row = 8 - row!(m.from);
        let to_col = std::char::from_u32_unchecked(col!(m.to) as u32 + 'a' as u32);
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
}

/// print_board() prints the board

pub fn print_board(d: &Data) {
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
                let dark_u32 = light_char as u32 + 'a' as u32 - 'A' as u32;
                unsafe {
                    print!(" {}", std::char::from_u32_unchecked(dark_u32));
                }
            }
            _ => {}
        }
        if (i + 1) % 8 == 0 && i != 63 {
            print!("\n{} ", 7 - row!(i));
        }
    }
    print!("\n\n   a b c d e f g h\n\n");
}

/// print_result() checks to see if the game is over, and if so, prints the result.

pub fn print_result(d: &mut Data) {
    let mut i = 0;
    while i < d.first_move[1] {
        if board::makemove(d, d.gen_dat[i].m.bytes()) {
            board::takeback(d);
            break;
        }
        i += 1;
    }
    if i == d.first_move[1] {
        if board::in_check(d, d.side) {
            match d.side {
                LIGHT => println!("0-1 {{Black mates}}"),
                _ => println!("1-0 {{White mates}}"),
            }
        } else {
            println!("1/2-1/2 {{Stalemate}}");
        }
    } else if search::reps(d) == 2 {
        println!("1/2-1/2 {{Draw by repetition}}");
    } else if d.fifty >= 100 {
        println!("1/2-1/2 {{Draw by fifty move rule}}");
    }
}
