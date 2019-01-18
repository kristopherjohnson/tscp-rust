// board.rs
// Tom Kerrigan's Simple Chess Program (TSCP)
//
// Copyright 1997 Tom Kerrigan
//
// Rust port by Kristopher Johnson

use crate::data::{
    CASTLE, COLOR, EP, FIFTY, FIRST_MOVE, HASH, HASH_EP, HASH_PIECE, HASH_SIDE, HPLY, INIT_COLOR,
    INIT_PIECE, MAILBOX, MAILBOX64, OFFSET, OFFSETS, PIECE, PLY, SIDE, SLIDE, XSIDE,
};
use crate::defs::{Int, C1, C8, DARK, E1, E8, EMPTY, G1, G8, KING, LIGHT, PAWN};

// #rust TODO The row! and col! macros should go into the defs module, but I
// can't figure out how to make that work.
macro_rules! row {
    ( $x:expr ) => {
        $x >> 3
    };
}
macro_rules! col {
    ( $x:expr ) => {
        $x & 7
    };
}

// #rust gen_push!(from, to, bits) downcasts the arguments to u8, avoiding the
// need for a lot of explicit "as u8" expressions in calls of gen_push().
macro_rules! gen_push {
    ( $from:expr, $to:expr, $bits:expr ) => {
        gen_push($from as u8, $to as u8, $bits as u8)
    };
}

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
unsafe fn hash_rand() -> Int {
    let mut r = 0;
    for _ in 0..32 {
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
    for i in 0..64 {
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

unsafe fn in_check(s: Int) -> bool {
    for i in 0..64 {
        if PIECE[i] == KING && COLOR[i] == s {
            return attack(i as Int, s ^ 1);
        }
    }
    std::panic!("in_check: shouldn't get here");
}

/// attack() returns TRUE if square sq is being attacked by side s and FALSE
/// otherwise.

unsafe fn attack(sq: Int, s: Int) -> bool {
    for i in 0..64 {
        if COLOR[i] == s {
            if PIECE[i] == PAWN {
                if s == LIGHT {
                    if col!(i) != 0 && (i as Int) - 9 == sq {
                        return true;
                    }
                    if col!(i) != 7 && (i as Int) - 7 == sq {
                        return true;
                    }
                } else {
                    if col!(i) != 0 && (i as Int) + 7 == sq {
                        return true;
                    }
                    if col!(i) != 7 && (i as Int) + 9 == sq {
                        return true;
                    }
                }
            } else {
                for j in 0..(OFFSETS[PIECE[i] as usize] as usize) {
                    let mut n = i as Int;
                    loop {
                        let m64 = MAILBOX64[n as usize];
                        let offset = OFFSET[PIECE[i] as usize][j];
                        n = MAILBOX[(m64 + offset) as usize];
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

/// gen() generates pseudo-legal moves for the current position.  It scans the
/// board to find friendly pieces and then determines what squares they attack.
/// When it finds a piece/square combination, it calls gen_push to put the move
/// on the "move stack."

pub unsafe fn gen() {
    // so far, we have no moves for the current ply
    FIRST_MOVE[PLY as usize + 1] = FIRST_MOVE[PLY as usize];

    for i in 0..64 {
        if COLOR[i] == SIDE {
            if PIECE[i] == PAWN {
                if SIDE == LIGHT {
                    if col!(i) != 0 && COLOR[i - 9] == DARK {
                        gen_push!(i, i - 9, 17);
                    }
                    if col!(i) != 7 && COLOR[i - 7] == DARK {
                        gen_push!(i, i - 7, 17);
                    }
                    if COLOR[i - 8] == EMPTY {
                        gen_push!(i, i - 8, 16);
                        if i >= 48 && COLOR[i - 16] == EMPTY {
                            gen_push!(i, i - 16, 24);
                        }
                    }
                } else {
                    if col!(i) != 0 && COLOR[i + 7] == LIGHT {
                        gen_push!(i, i + 7, 17);
                    }
                    if col!(i) != 7 && COLOR[i + 9] == LIGHT {
                        gen_push!(i, i + 9, 17);
                    }
                    if COLOR[i + 8] == EMPTY {
                        gen_push!(i, i + 8, 16);
                        if i <= 15 && COLOR[i + 16] == EMPTY {
                            gen_push!(i, i + 16, 24);
                        }
                    }
                }
            } else {
                for j in 0..(OFFSETS[PIECE[i] as usize] as usize) {
                    let mut n = i as i32;
                    loop {
                        let m64 = MAILBOX64[n as usize];
                        let offset = OFFSET[PIECE[i] as usize][j];
                        n = MAILBOX[(m64 + offset) as usize];
                        if n == -1 {
                            break;
                        }
                        let color = COLOR[n as usize];
                        if color != EMPTY {
                            if color == XSIDE {
                                gen_push!(i, n, 1);
                            }
                            break;
                        }
                        gen_push!(i, n, 0);
                        if !SLIDE[PIECE[i] as usize] {
                            break;
                        }
                    }
                }
            }
        }
    }

    // generate castle moves
    if SIDE == LIGHT {
        if (CASTLE & 1) != 0 {
            gen_push!(E1, G1, 2);
        }
        if (CASTLE & 2) != 0 {
            gen_push!(E1, C1, 2);
        }
    } else {
        if (CASTLE & 4) != 0 {
            gen_push!(E8, G8, 2);
        }
        if (CASTLE & 8) != 0 {
            gen_push!(E8, C8, 2);
        }
    }

    // generate en passant moves
    if EP != -1 {
        // #rust TODO Maybe there is a better way to avoid a bunch of "as usize"
        // casts in the expressions below.
        let i_ep = EP as usize;
        if SIDE == LIGHT {
            if col!(EP) != 0 && COLOR[i_ep + 7] == LIGHT && PIECE[i_ep + 7] == PAWN {
                gen_push!(EP + 7, EP, 21);
            }
            if col!(EP) != 7 && COLOR[i_ep + 9] == LIGHT && PIECE[i_ep + 9] == PAWN {
                gen_push!(EP + 9, EP, 21);
            }
        } else {
            if col!(EP) != 0 && COLOR[i_ep - 9] == DARK && PIECE[i_ep - 9] == PAWN {
                gen_push!(EP - 9, EP, 21);
            }
            if col!(EP) != 7 && COLOR[i_ep - 7] == DARK && PIECE[i_ep - 7] == PAWN {
                gen_push!(EP - 7, EP, 21);
            }
        }
    }
}

unsafe fn gen_push(from: u8, to: u8, bits: u8) {
    // #rust TODO
}
