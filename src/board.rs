//  board.rs
//  Tom Kerrigan's Simple Chess Program (TSCP)
//
//  Copyright 1997 Tom Kerrigan
//
//  Rust port by Kristopher Johnson

use crate::data::{
    CASTLE, COLOR, EP, FIFTY, FIRST_MOVE, HPLY, INIT_COLOR, INIT_PIECE, PIECE, PLY, SIDE, XSIDE,
};
use crate::defs::{DARK, LIGHT};

// init_board() sets the board to the initial game state.
pub unsafe fn init_board() {
    // #rust TODO: Can we just copy these arrays?
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
    //set_hash(); // init_hash() must be called
    FIRST_MOVE[0] = 0;
}
