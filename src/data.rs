//	data.rs
//	Tom Kerrigan's Simple Chess Program (TSCP)
//
//	Copyright 1997 Tom Kerrigan
//
//  Rust port by Kristopher Johnson

use crate::defs::{Gen, Hist, Move, GEN_STACK, HIST_STACK, MAX_PLY};

// the board representation

// LIGHT, DARK, or EMPTY
pub static mut COLOR: [i32; 64] = [0; 64];

// PAWN, KNIGHT, BISHOP, ROOK, QUEEN, KING, or EMPTY
pub static mut PIECE: [i32; 64] = [0; 64];

// the side to move
pub static mut SIDE: i32 = 0;

// the side not to move
pub static mut XSIDE: i32 = 0;

// a bitfield with the castle permissions. if 1 is set, white can still castle
// kingside. 2 is white queenside.  4 is black kingside. 8 is black queenside.
pub static mut CASTLE: i32 = 0;

// the en passant square. if white moves e2e4, the en passant square is set to
// e3, because that's where a pawn would move in an en passant capture
pub static mut EP: i32 = 0;

// the number of moves since a capture or pawn move, used to handle the
// fifty-move-draw rule
pub static mut FIFTY: i32 = 0;

// a (more or less) unique number that corresponds to the position
static mut HASH: i32 = 0;

// the number of half-moves (ply) since the root of the search tree
pub static mut PLY: i32 = 0;

// h for history; the number of ply since the beginning of the game
pub static mut HPLY: i32 = 0;

// GEN_DAT is some memory for move lists that are created by the move
// generators. The move list for ply n starts at FIRST_MOVE[n] and ends at
// FIRST_MOVE[n + 1].
static mut GEN_DAT: [Gen; GEN_STACK] = [Gen {
    m: Move { u: 0 },
    score: 0,
}; GEN_STACK];
pub static mut FIRST_MOVE: [i32; MAX_PLY] = [0; MAX_PLY];

// the history heuristic array (used for move ordering)
static mut HISTORY: [[i32; 64]; 64] = [[0; 64]; 64];

// we need an array of hist_t's so we can take back the moves we make
static mut HIST_DAT: [Hist; HIST_STACK] = [Hist {
    m: Move { u: 0 },
    capture: 0,
    castle: 0,
    ep: 0,
    fifty: 0,
    hash: 0,
}; HIST_STACK];

// the engine will search for max_time milliseconds or until it finishes
// searching max_depth ply.
static mut MAX_TIME: i32 = 0;
static mut MAX_DEPTH: i32 = 0;

// the time when the engine starts searching, and when it should stop
static mut START_TIME: i32 = 0;
static mut STOP_TIME: i32 = 0;

// the number of nodes we've searched
static mut NODES: i32 = 0;

// a "triangular" PV array; for a good explanation of why a triangular array is
// needed, see "How Computers Play Chess" by Levy and Newborn.
static mut PV: [[Move; MAX_PLY]; MAX_PLY] = [[Move { u: 0 }; MAX_PLY]; MAX_PLY];
static mut PV_LENGTH: [i32; MAX_PLY] = [0; MAX_PLY];
static mut FOLLOW_PV: bool = false;

// random numbers used to compute hash; see set_hash() in board.rs.
// indexed by piece [color][type][square]
static mut HASH_PIECE: [[[i32; 64]; 6]; 2] = [[[0; 64]; 6]; 2];
static mut HASH_SIDE: i32 = 0;
static mut HASH_EP: [i32; 64] = [0; 64];

// Now we have the mailbox array, so called because it looks like a mailbox, at
// least according to Bob Hyatt. This is useful when we need to figure out what
// pieces can go where. Let's say we have a rook on square a4 (32) and we want
// to know if it can move one square to the left. We subtract 1, and we get 31
// (h5). The rook obviously can't move to h5, but we don't know that without
// doing a lot of annoying work. Sooooo, what we do is figure out a4's mailbox
// number, which is 61. Then we subtract 1 from 61 (60) and see what mailbox[60]
// is. In this case, it's -1, so it's out of bounds and we can forget it. You
// can see how mailbox[] is used in attack() in board.rs.

#[rustfmt::skip]
const MAILBOX: [i32; 120] = [
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
const MAILBOX64: [i32; 64] = [
    21, 22, 23, 24, 25, 26, 27, 28,
    31, 32, 33, 34, 35, 36, 37, 38,
    41, 42, 43, 44, 45, 46, 47, 48,
    51, 52, 53, 54, 55, 56, 57, 58,
    61, 62, 63, 64, 65, 66, 67, 68,
    71, 72, 73, 74, 75, 76, 77, 78,
    81, 82, 83, 84, 85, 86, 87, 88,
    91, 92, 93, 94, 95, 96, 97, 98
];

// SLIDE, OFFSETS, and OFFSET are basically the vectors that
// pieces can move in. If SLIDE for the piece is False, it can
// only move one square in any one direction. OFFSETS is the
// number of directions it can move in, and OFFSET is an array
// of the actual directions.

const SLIDE: [bool; 6] = [false, false, true, true, true, false];

const OFFSETS: [i32; 6] = [0, 8, 4, 4, 8, 8];

const OFFSET: [[i32; 8]; 6] = [
    [0, 0, 0, 0, 0, 0, 0, 0],
    [-21, -19, -12, -8, 8, 12, 19, 21],
    [-11, -9, 9, 11, 0, 0, 0, 0],
    [-10, -1, 1, 10, 0, 0, 0, 0],
    [-11, -10, -9, -1, 1, 9, 10, 11],
    [-11, -10, -9, -1, 1, 9, 10, 11],
];

// This is the CASTLE_MASK array. We can use it to determine the castling
// permissions after a move. What we do is logical-AND the CASTLE bits with the
// CASTLE_MASK bits for both of the move's squares. Let's say CASTLE is 1,
// meaning that white can still castle kingside. Now we play a move where the
// rook on h1 gets captured. We AND CASTLE with CASTLE_MASK[63], so we have
// 1&14, and CASTLE becomes 0 and white can't castle kingside anymore.

#[rustfmt::skip]
const CASTLE_MASK: [i32; 64] = [
     7, 15, 15, 15,  3, 15, 15, 11,
    15, 15, 15, 15, 15, 15, 15, 15,
    15, 15, 15, 15, 15, 15, 15, 15,
    15, 15, 15, 15, 15, 15, 15, 15,
    15, 15, 15, 15, 15, 15, 15, 15,
    15, 15, 15, 15, 15, 15, 15, 15,
    15, 15, 15, 15, 15, 15, 15, 15,
    13, 15, 15, 15, 12, 15, 15, 14
];

// the piece letters, for print_board()
const PIECE_CHAR: [char; 6] = ['P', 'N', 'B', 'R', 'Q', 'K'];

// the initial board state

#[rustfmt::skip]
pub const INIT_COLOR: [i32; 64] = [
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
pub const INIT_PIECE: [i32; 64] = [
    3, 1, 2, 4, 5, 2, 1, 3,
    0, 0, 0, 0, 0, 0, 0, 0,
    6, 6, 6, 6, 6, 6, 6, 6,
    6, 6, 6, 6, 6, 6, 6, 6,
    6, 6, 6, 6, 6, 6, 6, 6,
    6, 6, 6, 6, 6, 6, 6, 6,
    0, 0, 0, 0, 0, 0, 0, 0,
    3, 1, 2, 4, 5, 2, 1, 3
];
