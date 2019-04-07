// eval.rs
// Tom Kerrigan's Simple Chess Program (TSCP)
//
// Copyright 1997 Tom Kerrigan
//
// Rust port by Kristopher Johnson

use crate::data::Data;
use crate::defs::{Int, BISHOP, DARK, EMPTY, KING, KNIGHT, LIGHT, PAWN, ROOK};

const DOUBLED_PAWN_PENALTY: Int = 10;
const ISOLATED_PAWN_PENALTY: Int = 20;
const BACKWARDS_PAWN_PENALTY: Int = 8;
const PASSED_PAWN_BONUS: Int = 20;
const ROOK_SEMI_OPEN_FILE_BONUS: Int = 10;
const ROOK_OPEN_FILE_BONUS: Int = 15;
const ROOK_ON_SEVENTH_BONUS: Int = 20;

/// the values of the pieces
const PIECE_VALUE: [Int; 6] = [100, 300, 300, 500, 900, 0];

// The "pcsq" arrays are piece/square tables. They're values added to the
// material value of the piece based on the location of the piece.

#[rustfmt::skip]
const PAWN_PCSQ: [Int; 64] = [
    0,   0,   0,   0,   0,   0,   0,   0,
    5,  10,  15,  20,  20,  15,  10,   5,
    4,   8,  12,  16,  16,  12,   8,   4,
    3,   6,   9,  12,  12,   9,   6,   3,
    2,   4,   6,   8,   8,   6,   4,   2,
    1,   2,   3, -10, -10,   3,   2,   1,
    0,   0,   0, -40, -40,   0,   0,   0,
    0,   0,   0,   0,   0,   0,   0,   0
];

#[rustfmt::skip]
const KNIGHT_PCSQ: [Int; 64] = [
    -10, -10, -10, -10, -10, -10, -10, -10,
    -10,   0,   0,   0,   0,   0,   0, -10,
    -10,   0,   5,   5,   5,   5,   0, -10,
    -10,   0,   5,  10,  10,   5,   0, -10,
    -10,   0,   5,  10,  10,   5,   0, -10,
    -10,   0,   5,   5,   5,   5,   0, -10,
    -10,   0,   0,   0,   0,   0,   0, -10,
    -10, -30, -10, -10, -10, -10, -30, -10
];

#[rustfmt::skip]
const BISHOP_PCSQ: [Int; 64] = [
    -10, -10, -10, -10, -10, -10, -10, -10,
    -10,   0,   0,   0,   0,   0,   0, -10,
    -10,   0,   5,   5,   5,   5,   0, -10,
    -10,   0,   5,  10,  10,   5,   0, -10,
    -10,   0,   5,  10,  10,   5,   0, -10,
    -10,   0,   5,   5,   5,   5,   0, -10,
    -10,   0,   0,   0,   0,   0,   0, -10,
    -10, -10, -20, -10, -10, -20, -10, -10
];

#[rustfmt::skip]
const KING_PCSQ: [Int; 64] = [
    -40, -40, -40, -40, -40, -40, -40, -40,
    -40, -40, -40, -40, -40, -40, -40, -40,
    -40, -40, -40, -40, -40, -40, -40, -40,
    -40, -40, -40, -40, -40, -40, -40, -40,
    -40, -40, -40, -40, -40, -40, -40, -40,
    -40, -40, -40, -40, -40, -40, -40, -40,
    -20, -20, -20, -20, -20, -20, -20, -20,
      0,  20,  40, -20,   0, -20,  40,  20
];

#[rustfmt::skip]
const KING_ENDGAME_PCSQ: [Int; 64] = [
     0,  10,  20,  30,  30,  20,  10,   0,
    10,  20,  30,  40,  40,  30,  20,  10,
    20,  30,  40,  50,  50,  40,  30,  20,
    30,  40,  50,  60,  60,  50,  40,  30,
    30,  40,  50,  60,  60,  50,  40,  30,
    20,  30,  40,  50,  50,  40,  30,  20,
    10,  20,  30,  40,  40,  30,  20,  10,
     0,  10,  20,  30,  30,  20,  10,   0
];

/// The FLIP array is used to calculate the piece/square values for DARK pieces.
/// The piece/square value of a LIGHT pawn is PAWN_PCSQ[sq] and the value of a
/// DARK pawn is PAWN_PCSQ[FLIP[sq]]
#[rustfmt::skip]
const FLIP: [usize; 64] = [
    56,  57,  58,  59,  60,  61,  62,  63,
    48,  49,  50,  51,  52,  53,  54,  55,
    40,  41,  42,  43,  44,  45,  46,  47,
    32,  33,  34,  35,  36,  37,  38,  39,
    24,  25,  26,  27,  28,  29,  30,  31,
    16,  17,  18,  19,  20,  21,  22,  23,
     8,   9,  10,  11,  12,  13,  14,  15,
     0,   1,   2,   3,   4,   5,   6,   7
];

pub fn eval(d: &mut Data) -> Int {
    let mut score = [0; 2];

    // this is the first pass: set up d.pawn_rank, d.piece_mat, and d.pawn_mat
    for i in 0..10 {
        d.pawn_rank[LIGHT as usize][i] = 0;
        d.pawn_rank[DARK as usize][i] = 7;
    }
    d.piece_mat[LIGHT as usize] = 0;
    d.piece_mat[DARK as usize] = 0;
    d.pawn_mat[LIGHT as usize] = 0;
    d.pawn_mat[DARK as usize] = 0;
    for i in 0..64 {
        if d.color[i] == EMPTY {
            continue;
        }
        if d.piece[i] == PAWN {
            d.pawn_mat[d.color[i] as usize] += PIECE_VALUE[PAWN as usize];
            let f = col!(i) + 1; // add 1 because of the extra file in the array
            if d.color[i] == LIGHT {
                if d.pawn_rank[LIGHT as usize][f] < row!(i as Int) {
                    d.pawn_rank[LIGHT as usize][f] = row!(i as Int);
                }
            } else if d.pawn_rank[DARK as usize][f] > row!(i as Int) {
                d.pawn_rank[DARK as usize][f] = row!(i as Int);
            }
        } else {
            d.piece_mat[d.color[i] as usize] +=
                PIECE_VALUE[d.piece[i] as usize];
        }
    }

    // this is the second pass: evaluate each piece
    score[LIGHT as usize] =
        d.piece_mat[LIGHT as usize] + d.pawn_mat[LIGHT as usize];
    score[DARK as usize] =
        d.piece_mat[DARK as usize] + d.pawn_mat[DARK as usize];
    for i in 0..64 {
        if d.color[i] == EMPTY {
            continue;
        }
        if d.color[i] == LIGHT {
            match d.piece[i] {
                PAWN => {
                    score[LIGHT as usize] += eval_light_pawn(d, i);
                }
                KNIGHT => {
                    score[LIGHT as usize] += KNIGHT_PCSQ[i];
                }
                BISHOP => {
                    score[LIGHT as usize] += BISHOP_PCSQ[i];
                }
                ROOK => {
                    if d.pawn_rank[LIGHT as usize][col!(i) + 1] == 0 {
                        if d.pawn_rank[DARK as usize][col!(i) + 1] == 7 {
                            score[LIGHT as usize] += ROOK_OPEN_FILE_BONUS;
                        } else {
                            score[LIGHT as usize] += ROOK_SEMI_OPEN_FILE_BONUS;
                        }
                    }
                    if row!(i) == 1 {
                        score[LIGHT as usize] += ROOK_ON_SEVENTH_BONUS;
                    }
                }
                KING => {
                    if d.piece_mat[DARK as usize] <= 1200 {
                        score[LIGHT as usize] += KING_ENDGAME_PCSQ[i];
                    } else {
                        score[LIGHT as usize] += eval_light_king(d, i);
                    }
                }
                _ => {}
            }
        } else {
            match d.piece[i] {
                PAWN => {
                    score[DARK as usize] += eval_dark_pawn(d, i);
                }
                KNIGHT => {
                    score[DARK as usize] += KNIGHT_PCSQ[FLIP[i]];
                }
                BISHOP => {
                    score[DARK as usize] += BISHOP_PCSQ[FLIP[i]];
                }
                ROOK => {
                    if d.pawn_rank[DARK as usize][col!(i) + 1] == 7 {
                        if d.pawn_rank[LIGHT as usize][col!(i) + 1] == 0 {
                            score[DARK as usize] += ROOK_OPEN_FILE_BONUS;
                        } else {
                            score[DARK as usize] += ROOK_SEMI_OPEN_FILE_BONUS;
                        }
                    }
                    if row!(i) == 6 {
                        score[DARK as usize] += ROOK_ON_SEVENTH_BONUS;
                    }
                }
                KING => {
                    if d.piece_mat[LIGHT as usize] <= 1200 {
                        score[DARK as usize] += KING_ENDGAME_PCSQ[FLIP[i]];
                    } else {
                        score[DARK as usize] += eval_dark_king(d, i);
                    }
                }
                _ => {}
            }
        }
    }

    // the score[] array is set, now return the score relative to the side to
    // move
    if d.side == LIGHT {
        score[LIGHT as usize] - score[DARK as usize]
    } else {
        score[DARK as usize] - score[LIGHT as usize]
    }
}

fn eval_light_pawn(d: &Data, sq: usize) -> Int {
    // the value to return
    let mut r = 0;

    // the pawn's file
    let f = (col!(sq as Int) + 1) as usize;

    // the pawn's row
    let row = row!(sq as Int);

    r += PAWN_PCSQ[sq];

    // if there's a pawn behind this one, it's doubled
    if d.pawn_rank[LIGHT as usize][f] > row {
        r -= DOUBLED_PAWN_PENALTY;
    }

    // if there aren't any friendly pawns on either side of this one, it's
    // isolated
    if (d.pawn_rank[LIGHT as usize][f - 1] == 0)
        && (d.pawn_rank[LIGHT as usize][f + 1] == 0)
    {
        r -= ISOLATED_PAWN_PENALTY;
    }
    // if it's not isolated, it might be backwards
    else if (d.pawn_rank[LIGHT as usize][f - 1] < row)
        && (d.pawn_rank[LIGHT as usize][f + 1] < row)
    {
        r -= BACKWARDS_PAWN_PENALTY;
    }

    // add a bonus if the pawn is passed
    if (d.pawn_rank[DARK as usize][f - 1] >= row)
        && (d.pawn_rank[DARK as usize][f] >= row)
        && (d.pawn_rank[DARK as usize][f + 1] >= row)
    {
        r += (7 - row) * PASSED_PAWN_BONUS;
    }

    r
}

fn eval_dark_pawn(d: &Data, sq: usize) -> Int {
    // the value to return
    let mut r = 0;

    // the pawn's file
    let f = (col!(sq as Int) + 1) as usize;

    // the pawn's row
    let row = row!(sq as Int);

    r += PAWN_PCSQ[FLIP[sq]];

    // if there's a pawn behind this one, it's doubled
    if d.pawn_rank[DARK as usize][f] < row {
        r -= DOUBLED_PAWN_PENALTY;
    }

    // if there aren't any friendly pawns on either side of this one, it's
    // isolated
    if (d.pawn_rank[DARK as usize][f - 1] == 7)
        && (d.pawn_rank[DARK as usize][f + 1] == 7)
    {
        r -= ISOLATED_PAWN_PENALTY;
    }
    // if it's not isolated, it might be backwards
    else if (d.pawn_rank[DARK as usize][f - 1] > row)
        && (d.pawn_rank[DARK as usize][f + 1] > row)
    {
        r -= BACKWARDS_PAWN_PENALTY;
    }

    // add a bonus if the pawn is passed
    if (d.pawn_rank[LIGHT as usize][f - 1] <= row)
        && (d.pawn_rank[LIGHT as usize][f] <= row)
        && (d.pawn_rank[LIGHT as usize][f + 1] <= row)
    {
        r += row * PASSED_PAWN_BONUS;
    }

    r
}

fn eval_light_king(d: &Data, sq: usize) -> Int {
    // the value to return
    let mut r = KING_PCSQ[sq];

    let col = col!(sq as Int);

    // if the king is castled, use a special function to evaluate the pawns on
    // the appropriate side
    if col < 3 {
        r += eval_lkp(d, 1);
        r += eval_lkp(d, 2);
        r += eval_lkp(d, 3) / 2; // problems with pawns on the c & f files are not as severe
    } else if col > 4 {
        r += eval_lkp(d, 8);
        r += eval_lkp(d, 7);
        r += eval_lkp(d, 6) / 2;
    }
    // otherwise just assess a penalty if there are open files near the king
    else {
        for i in (col as usize)..=(col as usize + 2) {
            if (d.pawn_rank[LIGHT as usize][i] == 0)
                && (d.pawn_rank[DARK as usize][i] == 7)
            {
                r -= 10;
            }
        }
    }

    // scale the king safely value according to the opponent's material; the
    // premise is that your king safety can only be bad if the opponent has
    // enough pieces to attack you.
    r *= d.piece_mat[DARK as usize];
    r /= 3100;

    r
}

/// eval_lkp(f) evaluates the Light King Pawn on file f

fn eval_lkp(d: &Data, f: usize) -> Int {
    let mut r = 0;

    let rank_light = d.pawn_rank[LIGHT as usize][f];

    if rank_light == 6 {
        // pawn hasn't moved
    } else if rank_light == 5 {
        r -= 10; // pawn moved one square
    } else if rank_light != 0 {
        r -= 20; // pawn moved more than one square
    } else {
        r -= 25; // no pawn on this file
    }

    let rank_dark = d.pawn_rank[DARK as usize][f];

    if rank_dark == 7 {
        r -= 15; // no enemy pawn
    } else if rank_dark == 5 {
        r -= 10; // enemy pawn on the 3rd rank
    } else if rank_dark == 4 {
        r -= 5; // enemy pawn on the 4th rank
    }

    r
}

fn eval_dark_king(d: &Data, sq: usize) -> Int {
    let mut r = KING_PCSQ[FLIP[sq]];

    let col = col!(sq as Int);

    if col < 3 {
        r += eval_dkp(d, 1);
        r += eval_dkp(d, 2);
        r += eval_dkp(d, 3) / 2;
    } else if col > 4 {
        r += eval_dkp(d, 8);
        r += eval_dkp(d, 7);
        r += eval_dkp(d, 6) / 2;
    } else {
        for i in (col as usize)..=(col as usize + 2) {
            if (d.pawn_rank[LIGHT as usize][i] == 0)
                && (d.pawn_rank[DARK as usize][i] == 7)
            {
                r -= 10;
            }
        }
    }
    r *= d.piece_mat[LIGHT as usize];
    r /= 3100;
    r
}

fn eval_dkp(d: &Data, f: usize) -> Int {
    let mut r = 0;

    let rank_dark = d.pawn_rank[DARK as usize][f];

    if rank_dark == 1 {
        ;
    } else if rank_dark == 2 {
        r -= 10;
    } else if rank_dark != 7 {
        r -= 20;
    } else {
        r -= 25;
    }

    let rank_light = d.pawn_rank[LIGHT as usize][f];

    if rank_light == 0 {
        r -= 15;
    } else if rank_light == 2 {
        r -= 10;
    } else if rank_light == 3 {
        r -= 5;
    }

    r
}
