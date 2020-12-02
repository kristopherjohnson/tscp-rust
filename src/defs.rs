// defs.rs
// Tom Kerrigan's Simple Chess Program (TSCP)
//
// Copyright 1997 Tom Kerrigan
//
// Rust port by Kristopher Johnson

pub type Int = i64;

pub const GEN_STACK: usize = 1120;
pub const MAX_PLY: usize = 32;
pub const HIST_STACK: usize = 400;

pub const LIGHT: Int = 0;
pub const DARK: Int = 1;

pub const PAWN: Int = 0;
pub const KNIGHT: Int = 1;
pub const BISHOP: Int = 2;
pub const ROOK: Int = 3;
pub const QUEEN: Int = 4;
pub const KING: Int = 5;

pub const EMPTY: Int = 6;

// useful squares
pub const A1: usize = 56;
pub const B1: usize = 57;
pub const C1: usize = 58;
pub const D1: usize = 59;
pub const E1: usize = 60;
pub const F1: usize = 61;
pub const G1: usize = 62;
pub const H1: usize = 63;
pub const A8: usize = 0;
pub const B8: usize = 1;
pub const C8: usize = 2;
pub const D8: usize = 3;
pub const E8: usize = 4;
pub const F8: usize = 5;
pub const G8: usize = 6;
pub const H8: usize = 7;

/// Get the row number for a square
macro_rules! row {
    ( $x:expr ) => {
        $x >> 3
    };
}

/// Get the column index for a square
macro_rules! col {
    ( $x:expr ) => {
        $x & 7
    };
}

/// This is the basic description of a move. promote is what
/// piece to promote the pawn to, if the move is a pawn
/// promotion. bits is a bitfield that describes the move,
/// with the following bits:
///
/// - 1  capture
/// - 2  castle
/// - 4  en passant capture
/// - 8  pushing a pawn 2 squares
/// - 16 pawn move
/// - 32 promote
///
/// It's union'ed with an integer so two moves can easily
/// be compared with each other.

#[derive(Copy, Clone, Default)]
pub struct MoveBytes {
    pub from: u8,
    pub to: u8,
    pub promote: u8,
    pub bits: u8,
}

// #rust TODO The use of a union for move representation requires a lot of code
// to be in "unsafe" blocks.  We provide safe functions to avoid the need for
// that.

#[derive(Copy, Clone)]
pub union Move {
    pub b: MoveBytes,
    pub u: Int,
}

impl Move {
    /// safely extract MoveBytes variant from a Move
    #[inline(always)]
    pub fn bytes(self: Move) -> MoveBytes {
        unsafe { self.b }
    }

    /// safely set MoveBytes variant in a Move
    #[inline(always)]
    pub fn set_bytes(self: &mut Move, m: MoveBytes) {
        self.b = m
    }

    /// safely extract 32-bit value from a Move
    #[inline(always)]
    pub fn value(self: Move) -> Int {
        unsafe { self.u }
    }

    /// safely set 32-bit value for a Move
    #[inline(always)]
    pub fn set_value(self: &mut Move, value: Int) {
        self.u = value;
    }
}

impl Default for Move {
    fn default() -> Self {
        Move { u: 0 }
    }
}

/// an element of the move stack. it's just a move with a score, so it can be
/// sorted by the search functions.
#[derive(Copy, Clone, Default)]
pub struct Gen {
    pub m: Move,
    pub score: Int,
}

/// an element of the history stack, with the information necessary to take a
/// move back.
#[derive(Copy, Clone, Default)]
pub struct Hist {
    pub m: Move,
    pub capture: Int,
    pub castle: Int,
    pub ep: Int,
    pub fifty: Int,
    pub hash: Int,
}
