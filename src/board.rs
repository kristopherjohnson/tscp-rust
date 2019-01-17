//  board.rs
//  Tom Kerrigan's Simple Chess Program (TSCP)
//
//  Copyright 1997 Tom Kerrigan
//
//  Rust port by Kristopher Johnson

use crate::data::{
    CASTLE, COLOR, EP, FIFTY, FIRST_MOVE, HASH, HASH_EP, HASH_PIECE, HASH_SIDE, HPLY, INIT_COLOR,
    INIT_PIECE, MAILBOX, MAILBOX64, OFFSET, OFFSETS, PIECE, PLY, SIDE, SLIDE, XSIDE,
};
use crate::defs::{col, DARK, EMPTY, KING, LIGHT, PAWN};

/// init_board() sets the board to the initial game state.
pub unsafe fn init_board() {
    // #rust TODO: Can we just copy these arrays rather than copying
    // element-by-element?
    for i in 0..64 {
        COLOR[i] = INIT_COLOR[i];
        PIECE[i] = INIT_PIECE[i];
    }
    SIDE = LIGHT;
    XSIDE = DARK;
    CASTLE = 15;
    EP = -1;
    FIFTY = 0;
    PLY = 0;
    HPLY = 0;
    set_hash(); // init_hash() must be called
    FIRST_MOVE[0] = 0;
}

/// init_hash() initializes the random numbers used by set_hash().

pub unsafe fn init_hash() {
    libc::srand(0);
    for i in 0..2 {
        for j in 0..6 {
            for k in 0..64 {
                HASH_PIECE[i][j][k] = hash_rand();
            }
        }
    }
    HASH_SIDE = hash_rand();
    for i in 0..64 {
        HASH_EP[i] = hash_rand();
    }
}

/// hash_rand() XORs some shifted random numbers together to make sure
/// we have good coverage of all 32 bits. (rand() returns 16-bit numbers
/// on some systems.)
unsafe fn hash_rand() -> i32 {
    let mut r: i32 = 0;
    for i in 0..32 {
        r ^= libc::rand() << 1;
    }
    r
}

/// set_hash() uses the Zobrist method of generating a unique number (hash)
/// for the current chess position. Of course, there are many more chess
/// positions than there are 32 bit numbers, so the numbers generated are
/// not really unique, but they're unique enough for our purposes (to detect
/// repetitions of the position).
/// The way it works is to XOR random numbers that correspond to features of
/// the position, e.g., if there's a black knight on B8, hash is XORed with
/// hash_piece[BLACK][KNIGHT][B8]. All of the pieces are XORed together,
/// hash_side is XORed if it's black's move, and the en passant square is
/// XORed if there is one. (A chess technicality is that one position can't
/// be a repetition of another if the en passant state is different.)

unsafe fn set_hash() {
    HASH = 0;
    for i in 0..COLOR.len() {
        if COLOR[i] != EMPTY {
            HASH ^= HASH_PIECE[COLOR[i] as usize][PIECE[i] as usize][i as usize];
        }
    }
    if SIDE == DARK {
        HASH ^= HASH_SIDE;
    }
    if EP != -1 {
        HASH ^= HASH_EP[EP as usize];
    }
}

/// in_check() returns TRUE if side s is in check and FALSE otherwise. It just
/// scans the board to find side s's king and calls attack() to see if it's
/// being attacked.

unsafe fn in_check(s: i32) -> bool {
    for i in 0..64 {
        if PIECE[i] == KING && COLOR[i] == s {
            return attack(i as i32, s ^ 1);
        }
    }
    std::panic!("in_check: shouldn't get here");
}

/// attack() returns TRUE if square sq is being attacked by side s and FALSE
/// otherwise.

unsafe fn attack(sq: i32, s: i32) -> bool {
    for i in 0..64 {
        if COLOR[i] == s {
            if PIECE[i] == PAWN {
                if s == LIGHT {
                    if col(i) != 0 && (i as i32) - 9 == sq {
                        return true;
                    }
                    if col(i) != 7 && (i as i32) - 7 == sq {
                        return true;
                    }
                } else {
                    if col(i) != 0 && (i as i32) + 7 == sq {
                        return true;
                    }
                    if col(i) != 7 && (i as i32) + 9 == sq {
                        return true;
                    }
                }
            } else {
                for j in 0..(OFFSETS[PIECE[i] as usize] as usize) {
                    let mut n = i as i32;
                    loop {
                        let m64 = MAILBOX64[n as usize] as usize;
                        let offset = OFFSET[PIECE[i] as usize][j] as usize;
                        n = MAILBOX[m64 + offset];
                        if n == -1 {
                            break;
                        }
                        if n == sq {
                            return true;
                        }
                        if COLOR[n as usize] != EMPTY {
                            break;
                        }
                        if !SLIDE[PIECE[i] as usize] {
                            break;
                        }
                    }
                }
            }
        }
    }
    false
}
