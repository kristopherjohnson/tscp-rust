// data.rs
// Tom Kerrigan's Simple Chess Program (TSCP)
//
// Copyright 1997 Tom Kerrigan
//
// Rust port by Kristopher Johnson

use crate::defs::{
    Gen, Hist, Int, Move, DARK, GEN_STACK, HIST_STACK, LIGHT, MAX_PLY,
};

// #rustc In the original C code, all the elements of the Data struct below are
// global variables.  In Rust, we wrap them all in a struct so that we don't
// have to treat them as "unsafe" static mutable data.

/// the board representation

pub struct Data {
    /// LIGHT, DARK, or EMPTY
    pub color: [Int; 64],

    /// PAWN, KNIGHT, BISHOP, ROOK, QUEEN, KING, or EMPTY
    pub piece: [Int; 64],

    /// the side to move
    pub side: Int,

    /// the side not to move
    pub xside: Int,

    /// a bitfield with the castle permissions. if 1 is set, white can still
    /// castle kingside. 2 is white queenside.  4 is black kingside. 8 is black
    /// queenside.
    pub castle: Int,

    /// the en passant square. if white moves e2e4, the en passant square is set
    /// to e3, because that's where a pawn would move in an en passant capture
    pub ep: Int,

    /// the number of moves since a capture or pawn move, used to handle the
    /// fifty-move-draw rule
    pub fifty: Int,

    /// a (more or less) unique number that corresponds to the position
    pub hash: Int,

    /// the number of half-moves (ply) since the root of the search tree
    pub ply: usize,

    /// h for history; the number of ply since the beginning of the game
    pub hply: usize,

    /// GEN_DAT is some memory for move lists that are created by the move
    /// generators. The move list for ply n starts at FIRST_MOVE[n] and ends at
    /// FIRST_MOVE[n + 1].
    pub gen_dat: [Gen; GEN_STACK],
    pub first_move: [usize; MAX_PLY],

    /// the history heuristic array (used for move ordering)
    pub history: [[Int; 64]; 64],

    /// we need an array of hist_t's so we can take back the moves we make
    pub hist_dat: [Hist; HIST_STACK],
    /// the engine will search for max_time milliseconds or until it finishes
    /// searching max_depth ply.
    pub max_time: Int,
    pub max_depth: Int,

    /// the time when the engine starts searching, and when it should stop
    pub start_time: u128,
    pub stop_time: u128,

    /// the number of nodes we've searched
    pub nodes: Int,

    /// a "triangular" PV array; for a good explanation of why a triangular
    /// array is needed, see "How Computers Play Chess" by Levy and Newborn.
    pub pv: [[Move; MAX_PLY]; MAX_PLY],
    pub pv_length: [usize; MAX_PLY],
    pub follow_pv: bool,

    /// random numbers used to compute hash; see set_hash() in board.rs.
    /// indexed by piece [color][type][square]
    pub hash_piece: [[[Int; 64]; 6]; 2],
    pub hash_side: Int,
    pub hash_ep: [Int; 64],

    /// opening book
    pub book_lines: Vec<String>,

    /// pawn_rank[x][y] is the rank of the least advanced pawn of color x on
    /// file y - 1. There are "buffer files" on the left and right to avoid
    /// special-case logic later. If there's no pawn on a rank, we pretend the
    /// pawn is impossibly far advanced (0 for LIGHT and 7 for DARK).  This
    /// makes it easy to test for pawns on a rank and it simplifies some pawn
    /// evaluation code.
    pub pawn_rank: [[Int; 10]; 2],

    /// the value of a side's pieces
    pub piece_mat: [Int; 2],

    /// the value of a side's pawns
    pub pawn_mat: [Int; 2],
}

impl Data {
    /// create a new instance of Data

    pub fn new() -> Data {
        Data {
            color: INIT_COLOR,
            piece: INIT_PIECE,
            side: LIGHT,
            xside: DARK,
            castle: 15,
            ep: -1,
            fifty: 0,
            hash: 0,
            ply: 0,
            hply: 0,
            gen_dat: [Gen {
                m: Move::default(),
                score: 0,
            }; GEN_STACK],
            first_move: [0; MAX_PLY],
            history: [[0; 64]; 64],
            hist_dat: [Hist {
                m: Move::default(),
                capture: 0,
                castle: 0,
                ep: 0,
                fifty: 0,
                hash: 0,
            }; HIST_STACK],
            max_time: 0,
            max_depth: 0,
            start_time: 0,
            stop_time: 0,
            nodes: 0,
            pv: [[Move::default(); MAX_PLY]; MAX_PLY],
            pv_length: [0; MAX_PLY],
            follow_pv: false,
            hash_piece: [[[0; 64]; 6]; 2],
            hash_side: 0,
            hash_ep: [0; 64],
            book_lines: Vec::new(),
            pawn_rank: [[0; 10]; 2],
            piece_mat: [0; 2],
            pawn_mat: [0; 2],
        }
    }
}

impl Default for Data {
    fn default() -> Self {
        Self::new()
    }
}

/// Now we have the mailbox array, so called because it looks like a mailbox, at
/// least according to Bob Hyatt. This is useful when we need to figure out what
/// pieces can go where. Let's say we have a rook on square a4 (32) and we want
/// to know if it can move one square to the left. We subtract 1, and we get 31
/// (h5). The rook obviously can't move to h5, but we don't know that without
/// doing a lot of annoying work. Sooooo, what we do is figure out a4's mailbox
/// number, which is 61. Then we subtract 1 from 61 (60) and see what mailbox[60]
/// is. In this case, it's -1, so it's out of bounds and we can forget it. You
/// can see how mailbox[] is used in attack() in board.rs.

#[rustfmt::skip]
pub const MAILBOX: [Int; 120] = [
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1,  0,  1,  2,  3,  4,  5,  6,  7, -1,
    -1,  8,  9, 10, 11, 12, 13, 14, 15, -1,
    -1, 16, 17, 18, 19, 20, 21, 22, 23, -1,
    -1, 24, 25, 26, 27, 28, 29, 30, 31, -1,
    -1, 32, 33, 34, 35, 36, 37, 38, 39, -1,
    -1, 40, 41, 42, 43, 44, 45, 46, 47, -1,
    -1, 48, 49, 50, 51, 52, 53, 54, 55, -1,
    -1, 56, 57, 58, 59, 60, 61, 62, 63, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1
];

#[rustfmt::skip]
pub const MAILBOX64: [Int; 64] = [
    21, 22, 23, 24, 25, 26, 27, 28,
    31, 32, 33, 34, 35, 36, 37, 38,
    41, 42, 43, 44, 45, 46, 47, 48,
    51, 52, 53, 54, 55, 56, 57, 58,
    61, 62, 63, 64, 65, 66, 67, 68,
    71, 72, 73, 74, 75, 76, 77, 78,
    81, 82, 83, 84, 85, 86, 87, 88,
    91, 92, 93, 94, 95, 96, 97, 98
];

/// SLIDE, OFFSETS, and OFFSET are basically the vectors that
/// pieces can move in. If SLIDE for the piece is False, it can
/// only move one square in any one direction. OFFSETS is the
/// number of directions it can move in, and OFFSET is an array
/// of the actual directions.

pub const SLIDE: [bool; 6] = [false, false, true, true, true, false];

pub const OFFSETS: [Int; 6] = [0, 8, 4, 4, 8, 8];

pub const OFFSET: [[Int; 8]; 6] = [
    [0, 0, 0, 0, 0, 0, 0, 0],
    [-21, -19, -12, -8, 8, 12, 19, 21],
    [-11, -9, 9, 11, 0, 0, 0, 0],
    [-10, -1, 1, 10, 0, 0, 0, 0],
    [-11, -10, -9, -1, 1, 9, 10, 11],
    [-11, -10, -9, -1, 1, 9, 10, 11],
];

/// This is the CASTLE_MASK array. We can use it to determine the castling
/// permissions after a move. What we do is logical-AND the CASTLE bits with the
/// CASTLE_MASK bits for both of the move's squares. Let's say CASTLE is 1,
/// meaning that white can still castle kingside. Now we play a move where the
/// rook on h1 gets captured. We AND CASTLE with CASTLE_MASK[63], so we have
/// 1&14, and CASTLE becomes 0 and white can't castle kingside anymore.

#[rustfmt::skip]
pub const CASTLE_MASK: [Int; 64] = [
     7, 15, 15, 15,  3, 15, 15, 11,
    15, 15, 15, 15, 15, 15, 15, 15,
    15, 15, 15, 15, 15, 15, 15, 15,
    15, 15, 15, 15, 15, 15, 15, 15,
    15, 15, 15, 15, 15, 15, 15, 15,
    15, 15, 15, 15, 15, 15, 15, 15,
    15, 15, 15, 15, 15, 15, 15, 15,
    13, 15, 15, 15, 12, 15, 15, 14
];

/// the piece letters, for print_board()
pub const PIECE_CHAR: [char; 6] = ['P', 'N', 'B', 'R', 'Q', 'K'];

// the initial board state

#[rustfmt::skip]
pub const INIT_COLOR: [Int; 64] = [
    1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1,
    6, 6, 6, 6, 6, 6, 6, 6,
    6, 6, 6, 6, 6, 6, 6, 6,
    6, 6, 6, 6, 6, 6, 6, 6,
    6, 6, 6, 6, 6, 6, 6, 6,
    0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0
];

#[rustfmt::skip]
pub const INIT_PIECE: [Int; 64] = [
    3, 1, 2, 4, 5, 2, 1, 3,
    0, 0, 0, 0, 0, 0, 0, 0,
    6, 6, 6, 6, 6, 6, 6, 6,
    6, 6, 6, 6, 6, 6, 6, 6,
    6, 6, 6, 6, 6, 6, 6, 6,
    6, 6, 6, 6, 6, 6, 6, 6,
    0, 0, 0, 0, 0, 0, 0, 0,
    3, 1, 2, 4, 5, 2, 1, 3
];
