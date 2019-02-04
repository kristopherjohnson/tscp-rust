// board.rs
// Tom Kerrigan's Simple Chess Program (TSCP)
//
// Copyright 1997 Tom Kerrigan
//
// Rust port by Kristopher Johnson

use crate::data::{
    Data, CASTLE_MASK, INIT_COLOR, INIT_PIECE, MAILBOX, MAILBOX64, OFFSET,
    OFFSETS, SLIDE,
};
use crate::defs::{
    Int, MoveBytes, A1, A8, B1, B8, C1, C8, D1, D8, DARK, E1, E8, EMPTY, F1,
    F8, G1, G8, H1, H8, KING, KNIGHT, LIGHT, PAWN, QUEEN, ROOK,
};

// #rust gen_push!(d, from, to, bits) coerces the arguments to the right types,
// avoiding the need for a lot of explicit "as usize" and "as u8" coercions in
// calls to gen_push().
macro_rules! gen_push {
    ( $dref:ident, $from:expr, $to:expr, $bits:expr ) => {
        gen_push($dref, $from as usize, $to as usize, $bits as u8)
    };
}

/// init_board() sets the board to the initial game state.

pub fn init_board(d: &mut Data) {
    d.color = INIT_COLOR;
    d.piece = INIT_PIECE;
    d.side = LIGHT;
    d.xside = DARK;
    d.castle = 15;
    d.ep = -1;
    d.fifty = 0;
    d.ply = 0;
    d.hply = 0;
    set_hash(d); // init_hash() must be called
    d.first_move[0] = 0;
}

/// init_hash() initializes the random numbers used by set_hash().

pub fn init_hash(d: &mut Data) {
    unsafe {
        libc::srand(0);
    }
    for i in 0..2 {
        for j in 0..6 {
            for k in 0..64 {
                d.hash_piece[i][j][k] = hash_rand();
            }
        }
    }
    d.hash_side = hash_rand();
    for i in 0..64 {
        d.hash_ep[i] = hash_rand();
    }
}

/// hash_rand() XORs some shifted random numbers together to make sure
/// we have good coverage of all 32 bits. (rand() returns 16-bit numbers
/// on some systems.)
fn hash_rand() -> Int {
    let mut r: Int = 0;
    unsafe {
        for _ in 0..32 {
            r ^= (libc::rand() << 1) as Int;
        }
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

pub fn set_hash(d: &mut Data) {
    d.hash = 0;
    for i in 0..64 {
        if d.color[i] != EMPTY {
            d.hash ^= d.hash_piece[d.color[i] as usize][d.piece[i] as usize]
                [i as usize];
        }
    }
    if d.side == DARK {
        d.hash ^= d.hash_side;
    }
    if d.ep != -1 {
        d.hash ^= d.hash_ep[d.ep as usize];
    }
}

/// in_check() returns TRUE if side s is in check and FALSE otherwise. It just
/// scans the board to find side s's king and calls attack() to see if it's
/// being attacked.

pub fn in_check(d: &Data, s: Int) -> bool {
    for i in 0..64 {
        if d.piece[i] == KING && d.color[i] == s {
            return attack(&d, i, s ^ 1);
        }
    }
    panic!("in_check: shouldn't get here");
}

/// attack() returns true if square sq is being attacked by side s and false
/// otherwise.

fn attack(d: &Data, sq: usize, s: Int) -> bool {
    for i in 0..64 {
        if d.color[i] == s {
            if d.piece[i] == PAWN {
                if s == LIGHT {
                    if col!(i) != 0 && i - 9 == sq {
                        return true;
                    }
                    if col!(i) != 7 && i - 7 == sq {
                        return true;
                    }
                } else {
                    if col!(i) != 0 && i + 7 == sq {
                        return true;
                    }
                    if col!(i) != 7 && i + 9 == sq {
                        return true;
                    }
                }
            } else {
                for j in 0..(OFFSETS[d.piece[i] as usize] as usize) {
                    let mut n = i as Int;
                    loop {
                        let m64 = MAILBOX64[n as usize];
                        let offset = OFFSET[d.piece[i] as usize][j];
                        n = MAILBOX[(m64 + offset) as usize];
                        if n == -1 {
                            break;
                        }
                        if n as usize == sq {
                            return true;
                        }
                        if d.color[n as usize] != EMPTY {
                            break;
                        }
                        if !SLIDE[d.piece[i] as usize] {
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

pub fn gen(d: &mut Data) {
    // so far, we have no moves for the current ply
    d.first_move[d.ply + 1] = d.first_move[d.ply];

    for i in 0..64 {
        if d.color[i] == d.side {
            if d.piece[i] == PAWN {
                if d.side == LIGHT {
                    if col!(i) != 0 && d.color[i - 9] == DARK {
                        gen_push!(d, i, i - 9, 17);
                    }
                    if col!(i) != 7 && d.color[i - 7] == DARK {
                        gen_push!(d, i, i - 7, 17);
                    }
                    if d.color[i - 8] == EMPTY {
                        gen_push!(d, i, i - 8, 16);
                        if i >= 48 && d.color[i - 16] == EMPTY {
                            gen_push!(d, i, i - 16, 24);
                        }
                    }
                } else {
                    if col!(i) != 0 && d.color[i + 7] == LIGHT {
                        gen_push!(d, i, i + 7, 17);
                    }
                    if col!(i) != 7 && d.color[i + 9] == LIGHT {
                        gen_push!(d, i, i + 9, 17);
                    }
                    if d.color[i + 8] == EMPTY {
                        gen_push!(d, i, i + 8, 16);
                        if i <= 15 && d.color[i + 16] == EMPTY {
                            gen_push!(d, i, i + 16, 24);
                        }
                    }
                }
            } else {
                for j in 0..(OFFSETS[d.piece[i] as usize] as usize) {
                    let mut n = i as Int;
                    loop {
                        let m64 = MAILBOX64[n as usize];
                        let offset = OFFSET[d.piece[i] as usize][j];
                        n = MAILBOX[(m64 + offset) as usize];
                        if n == -1 {
                            break;
                        }
                        let color = d.color[n as usize];
                        if color != EMPTY {
                            if color == d.xside {
                                gen_push!(d, i, n, 1);
                            }
                            break;
                        }
                        gen_push!(d, i, n, 0);
                        if !SLIDE[d.piece[i] as usize] {
                            break;
                        }
                    }
                }
            }
        }
    }

    // generate castle moves
    if d.side == LIGHT {
        if (d.castle & 1) != 0 {
            gen_push!(d, E1, G1, 2);
        }
        if (d.castle & 2) != 0 {
            gen_push!(d, E1, C1, 2);
        }
    } else {
        if (d.castle & 4) != 0 {
            gen_push!(d, E8, G8, 2);
        }
        if (d.castle & 8) != 0 {
            gen_push!(d, E8, C8, 2);
        }
    }

    // generate en passant moves
    if d.ep != -1 {
        // #rust TODO Maybe there is a better way to avoid a bunch of "as usize"
        // casts in the expressions below.
        let i_ep = d.ep as usize;
        if d.side == LIGHT {
            if col!(d.ep) != 0
                && d.color[i_ep + 7] == LIGHT
                && d.piece[i_ep + 7] == PAWN
            {
                gen_push!(d, d.ep + 7, d.ep, 21);
            }
            if col!(d.ep) != 7
                && d.color[i_ep + 9] == LIGHT
                && d.piece[i_ep + 9] == PAWN
            {
                gen_push!(d, d.ep + 9, d.ep, 21);
            }
        } else {
            if col!(d.ep) != 0
                && d.color[i_ep - 9] == DARK
                && d.piece[i_ep - 9] == PAWN
            {
                gen_push!(d, d.ep - 9, d.ep, 21);
            }
            if col!(d.ep) != 7
                && d.color[i_ep - 7] == DARK
                && d.piece[i_ep - 7] == PAWN
            {
                gen_push!(d, d.ep - 7, d.ep, 21);
            }
        }
    }
}

/// gen_caps() is basically a copy of gen() that's modified to only generate
/// capture and promote moves. It's used by the quiescence search.

pub fn gen_caps(d: &mut Data) {
    d.first_move[d.ply + 1] = d.first_move[d.ply];
    for i in 0..64 {
        if d.color[i] == d.side {
            if d.piece[i] == PAWN {
                if d.side == LIGHT {
                    if col!(i) != 0 && d.color[i - 9] == DARK {
                        gen_push!(d, i, i - 9, 17);
                    }
                    if col!(i) != 7 && d.color[i - 7] == DARK {
                        gen_push!(d, i, i - 7, 17);
                    }
                    if i <= 15 && d.color[i - 8] == EMPTY {
                        gen_push!(d, i, i - 8, 16);
                    }
                }
                if d.side == DARK {
                    if col!(i) != 0 && d.color[i + 7] == LIGHT {
                        gen_push!(d, i, i + 7, 17);
                    }
                    if col!(i) != 7 && d.color[i + 9] == LIGHT {
                        gen_push!(d, i, i + 9, 17);
                    }
                    if i >= 48 && d.color[i + 8] == EMPTY {
                        gen_push!(d, i, i + 8, 16);
                    }
                }
            } else {
                for j in 0..(OFFSETS[d.piece[i] as usize] as usize) {
                    let mut n = i as Int;
                    loop {
                        let m64 = MAILBOX64[n as usize];
                        let offset = OFFSET[d.piece[i] as usize][j];
                        n = MAILBOX[(m64 + offset) as usize];
                        if n == -1 {
                            break;
                        }
                        let color = d.color[n as usize];
                        if color != EMPTY {
                            if color == d.xside {
                                gen_push!(d, i, n, 1);
                            }
                            break;
                        }
                        if !SLIDE[d.piece[i] as usize] {
                            break;
                        }
                    }
                }
            }
        }
    }

    if d.ep != -1 {
        // #rust TODO Maybe there is a better way to avoid a bunch of "as usize"
        // casts in the expressions below.
        let i_ep = d.ep as usize;
        if d.side == LIGHT {
            if col!(d.ep) != 0
                && d.color[i_ep + 7] == LIGHT
                && d.piece[i_ep + 7] == PAWN
            {
                gen_push!(d, d.ep + 7, d.ep, 21);
            }
            if col!(d.ep) != 7
                && d.color[i_ep + 9] == LIGHT
                && d.piece[i_ep + 9] == PAWN
            {
                gen_push!(d, d.ep + 9, d.ep, 21);
            }
        } else {
            if col!(d.ep) != 0
                && d.color[i_ep - 9] == DARK
                && d.piece[i_ep - 9] == PAWN
            {
                gen_push!(d, d.ep - 9, d.ep, 21);
            }
            if col!(d.ep) != 7
                && d.color[i_ep - 7] == DARK
                && d.piece[i_ep - 7] == PAWN
            {
                gen_push!(d, d.ep - 7, d.ep, 21);
            }
        }
    }
}

/// gen_push() puts a move on the move stack, unless it's a pawn promotion that
/// needs to be handled by gen_promote().  It also assigns a score to the move
/// for alpha-beta move ordering. If the move is a capture, it uses MVV/LVA
/// (Most Valuable Victim/Least Valuable Attacker). Otherwise, it uses the
/// move's history heuristic value. Note that 1,000,000 is added to a capture
/// move's score, so it always gets ordered above a "normal" move. */

fn gen_push(d: &mut Data, from: usize, to: usize, bits: u8) {
    if (bits & 16) != 0 {
        if d.side == LIGHT {
            if to <= H8 {
                gen_promote(d, from, to, bits);
                return;
            }
        } else {
            if to >= A1 {
                gen_promote(d, from, to, bits);
                return;
            }
        }
    }
    let g = &mut d.gen_dat[d.first_move[d.ply + 1] as usize];
    d.first_move[d.ply + 1] += 1;
    unsafe {
        g.m.b.from = from as u8;
        g.m.b.to = to as u8;
        g.m.b.promote = 0;
        g.m.b.bits = bits;
    }
    if d.color[to] != EMPTY {
        g.score = 1000000 + d.piece[to] * 10 - d.piece[from];
    } else {
        g.score = d.history[from][to];
    }
}

/// gen_promote() is just like gen_push(), only it puts 4 moves on the move
/// stack, one for each possible promotion piece

fn gen_promote(d: &mut Data, from: usize, to: usize, bits: u8) {
    for i in KNIGHT..=QUEEN {
        let g = &mut d.gen_dat[d.first_move[d.ply + 1] as usize];
        d.first_move[d.ply + 1] += 1;
        unsafe {
            g.m.b.from = from as u8;
            g.m.b.to = to as u8;
            g.m.b.promote = i as u8;
            g.m.b.bits = bits | 32;
        }
        g.score = 1000000 + (i * 10);
    }
}

/// makemove() makes a move. If the move is illegal, it
/// undoes whatever it did and returns FALSE. Otherwise, it
/// returns TRUE.

pub fn makemove(d: &mut Data, m: MoveBytes) -> bool {
    let from: usize;
    let to: usize;

    // test to see if a castle move is legal and move the rook (the king is
    // moved with the usual move code later)
    if (m.bits & 2) != 0 {
        if in_check(&d, d.side) {
            return false;
        }
        match m.to {
            62 => {
                if d.color[F1] != EMPTY
                    || d.color[G1] != EMPTY
                    || attack(&d, F1, d.xside)
                    || attack(&d, G1, d.xside)
                {
                    return false;
                }
                from = H1;
                to = F1;
            }
            58 => {
                if d.color[B1] != EMPTY
                    || d.color[C1] != EMPTY
                    || d.color[D1] != EMPTY
                    || attack(&d, C1, d.xside)
                    || attack(&d, D1, d.xside)
                {
                    return false;
                }
                from = A1;
                to = D1;
            }
            6 => {
                if d.color[F8] != EMPTY
                    || d.color[G8] != EMPTY
                    || attack(&d, F8, d.xside)
                    || attack(&d, G8, d.xside)
                {
                    return false;
                }
                from = H8;
                to = F8;
            }
            2 => {
                if d.color[B8] != EMPTY
                    || d.color[C8] != EMPTY
                    || d.color[D8] != EMPTY
                    || attack(&d, C8, d.xside)
                    || attack(&d, D8, d.xside)
                {
                    return false;
                }
                from = A8;
                to = D8;
            }
            _ => {
                panic!("makemove: invalid castling move");
            }
        }
        d.color[to] = d.color[from];
        d.piece[to] = d.piece[from];
        d.color[from] = EMPTY;
        d.piece[from] = EMPTY;
    }

    // back up information so we can take the move back later.
    d.hist_dat[d.hply].m.set_bytes(m);
    d.hist_dat[d.hply].capture = d.piece[m.to as usize];
    d.hist_dat[d.hply].castle = d.castle;
    d.hist_dat[d.hply].ep = d.ep;
    d.hist_dat[d.hply].fifty = d.fifty;
    d.hist_dat[d.hply].hash = d.hash;
    d.ply += 1;
    d.hply += 1;

    // update the castle, en passant, and fifty-move-draw variables
    d.castle &= CASTLE_MASK[m.from as usize] & CASTLE_MASK[m.to as usize];
    if (m.bits & 8) != 0 {
        if d.side == LIGHT {
            d.ep = m.to as Int + 8;
        } else {
            d.ep = m.to as Int - 8;
        }
    } else {
        d.ep = -1;
    }
    if (m.bits & 17) != 0 {
        d.fifty = 0;
    } else {
        d.fifty += 1;
    }

    // move the piece
    d.color[m.to as usize] = d.side;
    if (m.bits & 32) != 0 {
        d.piece[m.to as usize] = m.promote as Int;
    } else {
        d.piece[m.to as usize] = d.piece[m.from as usize];
    }
    d.color[m.from as usize] = EMPTY;
    d.piece[m.from as usize] = EMPTY;

    // erase the pawn if this is an en passant move
    if (m.bits & 4) != 0 {
        let pawn_sq =
            if d.side == LIGHT { m.to + 8 } else { m.to - 8 } as usize;
        d.color[pawn_sq] = EMPTY;
        d.piece[pawn_sq] = EMPTY;
    }

    // switch sides and test for legality (if we can capture the other guy's
    // king, it's an illegal position and we need to take the move back)
    d.side ^= 1;
    d.xside ^= 1;
    if in_check(&d, d.xside) {
        takeback(d);
        return false;
    }
    set_hash(d);
    true
}

/// takeback() is very similar to makemove(), only backwards :)

pub fn takeback(d: &mut Data) {
    d.side ^= 1;
    d.xside ^= 1;
    d.ply -= 1;
    d.hply -= 1;
    let m = d.hist_dat[d.hply].m.bytes();
    d.castle = d.hist_dat[d.hply].castle;
    d.ep = d.hist_dat[d.hply].ep;
    d.fifty = d.hist_dat[d.hply].fifty;
    d.hash = d.hist_dat[d.hply].hash;
    d.color[m.from as usize] = d.side;
    if (m.bits & 32) != 0 {
        d.piece[m.from as usize] = PAWN;
    } else {
        d.piece[m.from as usize] = d.piece[m.to as usize];
    }
    if d.hist_dat[d.hply].capture == EMPTY {
        d.color[m.to as usize] = EMPTY;
        d.piece[m.to as usize] = EMPTY;
    } else {
        d.color[m.to as usize] = d.xside;
        d.piece[m.to as usize] = d.hist_dat[d.hply].capture;
    }
    if (m.bits & 2) != 0 {
        let from: usize;
        let to: usize;
        match m.to as usize {
            G1 => {
                from = F1;
                to = H1;
            }
            C1 => {
                from = D1;
                to = A1;
            }
            G8 => {
                from = F8;
                to = H8;
            }
            C8 => {
                from = D8;
                to = A8;
            }
            _ => panic!("takeback: invalid castling move"),
        }
        d.color[to] = d.side;
        d.piece[to] = ROOK;
        d.color[from] = EMPTY;
        d.piece[from] = EMPTY;
    }
    if (m.bits & 4) != 0 {
        let pawn_sq =
            if d.side == LIGHT { m.to + 8 } else { m.to - 8 } as usize;
        d.color[pawn_sq] = d.xside;
        d.piece[pawn_sq] = PAWN;
    }
}
