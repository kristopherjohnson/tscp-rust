// scan.rs
// Tom Kerrigan's Simple Chess Program (TSCP)
//
// Copyright 1997 Tom Kerrigan
//
// Rust port by Kristopher Johnson

// #rust The original C code uses the C standard library's scanf("%s") to read
// tokens from the input, but Rust's standard library does not provide an
// analogous function.  This module provides a scan() function that is roughly
// equivalent.

use std::io;
use std::io::prelude::*;

/// reads a whitespace-delimited token from stdin. returns an empty string on
/// EOF. assumes input is 7-bit ASCII, and does not recognize Unicode whitespace
/// other than ' ', '\t', '\n', '\r', and '\v'.

pub fn scan() -> Result<String, io::Error> {
    let mut reader = io::stdin().lock();

    let mut bytes: Vec<u8> = Vec::new();

    // skip leading whitespace
    loop {
        match read_byte(&mut reader) {
            ReadByteResult::Ok(byte) => {
                if !is_whitespace(byte) {
                    bytes.push(byte);
                    break;
                }
            }
            ReadByteResult::Eof => {
                return Ok(String::new());
            }
            ReadByteResult::Err(err) => {
                return Err(err);
            }
        }
    }

    // copy bytes until whitespace or EOF
    loop {
        match read_byte(&mut reader) {
            ReadByteResult::Ok(byte) => {
                if is_whitespace(byte) {
                    break;
                }
                bytes.push(byte);
            }
            ReadByteResult::Eof => {
                break;
            }
            ReadByteResult::Err(err) => {
                return Err(err);
            }
        }
    }

    // convert bytes to a String
    let s = match String::from_utf8(bytes) {
        Ok(string) => string,
        Err(err) => return Err(io::Error::new(io::ErrorKind::InvalidData, err)),
    };
    Ok(s)
}

enum ReadByteResult {
    Ok(u8),
    Eof,
    Err(io::Error),
}

/// attempts to read a single byte from stdin.

fn read_byte(reader: &mut Read) -> ReadByteResult {
    let mut buffer: [u8; 1] = [0; 1];
    match reader.read(&mut buffer) {
        Ok(1) => ReadByteResult::Ok(buffer[0]),
        Ok(0) => ReadByteResult::Eof,
        Ok(other_size) => panic!("read_byte read {} bytes", other_size),
        Err(err) => ReadByteResult::Err(err),
    }
}

/// returns true if specified byte is an ASCII whitespace character

fn is_whitespace(ascii: u8) -> bool {
    match ascii {
        0x9 | 0xa | 0xb | 0xd | 0x20 => true,
        _ => false,
    }
}
