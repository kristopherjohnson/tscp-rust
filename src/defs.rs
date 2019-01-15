// defs.rs
// Tom Kerrigan's Simple Chess Program (TSCP)
//
// Copyright 1997 Tom Kerrigan
//
// Rust port by Kristopher Johnson

const GEN_STACK: u32 = 1120;
const MAX_PLY: u32 = 32;
const HIST_STACK: u32 = 400;

const LIGHT: u32 = 0;
const DARK: u32 = 1;

const PAWN: u32 = 0;
const KNIGHT: u32 = 1;
const BISHOP: u32 = 2;
const ROOK: u32 = 3;
const QUEEN: u32 = 4;
const KING: u32 = 5;

const EMPTY: u32 = 6;

// useful squares
const A1: u32 = 56;
const B1: u32 = 57;
const C1: u32 = 58;
const D1: u32 = 59;
const E1: u32 = 60;
const F1: u32 = 61;
const G1: u32 = 62;
const A8: u32 = 0;
const B8: u32 = 1;
const C8: u32 = 2;
const D8: u32 = 3;
const E8: u32 = 4;
const F8: u32 = 5;
const G8: u32 = 6;
const H8: u32 = 7;

// #rust TODO: Make these macros?
fn row(x: u32) -> u32 { x >> 3 }
fn col(x: u32) -> u32 { x & 7 }
