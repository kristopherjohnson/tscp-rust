//	defs.rs
//	Tom Kerrigan's Simple Chess Program (TSCP)
//
//	Copyright 1997 Tom Kerrigan
//
//  Rust port by Kristopher Johnson

pub const GEN_STACK: usize = 1120;
pub const MAX_PLY: usize = 32;
pub const HIST_STACK: usize = 400;

pub const LIGHT: i32 = 0;
pub const DARK: i32 = 1;

pub const PAWN: i32 = 0;
pub const KNIGHT: i32 = 1;
pub const BISHOP: i32 = 2;
pub const ROOK: i32 = 3;
pub const QUEEN: i32 = 4;
pub const KING: i32 = 5;

pub const EMPTY: i32 = 6;

// useful squares
pub const A1: i32 = 56;
pub const B1: i32 = 57;
pub const C1: i32 = 58;
pub const D1: i32 = 59;
pub const E1: i32 = 60;
pub const F1: i32 = 61;
pub const G1: i32 = 62;
pub const A8: i32 = 0;
pub const B8: i32 = 1;
pub const C8: i32 = 2;
pub const D8: i32 = 3;
pub const E8: i32 = 4;
pub const F8: i32 = 5;
pub const G8: i32 = 6;
pub const H8: i32 = 7;

// #rust TODO: Make these macros or generics?
pub fn row(square: i32) -> i32 {
    square >> 3
}
pub fn col(square: i32) -> i32 {
    square & 7
}

/* This is the basic description of a move. promote is what
piece to promote the pawn to, if the move is a pawn
promotion. bits is a bitfield that describes the move,
with the following bits:

1	capture
2	castle
4	en passant capture
8	pushing a pawn 2 squares
16	pawn move
32	promote

It's union'ed with an integer so two moves can easily
be compared with each other. */

#[derive(Copy, Clone)]
pub struct MoveBytes {
    pub from: u8,
    pub to: u8,
    pub promote: u8,
    pub bits: u8,
}

#[derive(Copy, Clone)]
pub union Move {
    pub b: MoveBytes,
    pub u: i32,
}

/* an element of the move stack. it's just a move with a
score, so it can be sorted by the search functions. */
#[derive(Copy, Clone)]
pub struct Gen {
    pub m: Move,
    pub score: i32,
}

/* an element of the history stack, with the information
necessary to take a move back. */
#[derive(Copy, Clone)]
pub struct Hist {
    pub m: Move,
    pub capture: i32,
    pub castle: i32,
    pub ep: i32,
    pub fifty: i32,
    pub hash: i32,
}
