// scan.rs
// Tom Kerrigan's Simple Chess Program (TSCP)
//
// Copyright 1997 Tom Kerrigan
//
// Rust port by Kristopher Johnson

// #rust The original C code uses the C standard library's scanf("%s") and
// scanf("%d") to read input values, but Rust's standard library does not
// provide an analogous function.  This module provides functions that are
// roughly equivalent.

use std::io;
use std::io::prelude::*;

use crate::defs::Int;

/// reads a whitespace-delimited token from stdin. returns an empty string on
/// EOF. assumes input is 7-bit ASCII, and does not recognize Unicode whitespace
/// other than ' ', '\t', '\n', '\r', and '\v'.
///
/// # Errors
///
/// Returns error if unable to read for a reason other than EOF.

pub fn scan_token() -> io::Result<String> {
    let stdin = io::stdin();
    let mut reader = stdin.lock();
    scan_token_from(&mut reader)
}

/// reads a whitespace-delimited token from a reader. returns an empty string on
/// EOF. assumes input is 7-bit ASCII, and does not recognize Unicode whitespace
/// other than ' ', '\t', '\n', '\r', and '\v'.
///
/// # Errors
///
/// Returns error if unable to read for a reason other than EOF.
///
/// # Example
/// ```
/// use tscp::scan::scan_token_from;
///
/// let input = String::from("  one   two three  ");
/// let mut bytes = input.as_bytes();
/// assert_eq!(scan_token_from(&mut bytes).unwrap(), "one");
/// assert_eq!(scan_token_from(&mut bytes).unwrap(), "two");
/// assert_eq!(scan_token_from(&mut bytes).unwrap(), "three");
/// assert_eq!(scan_token_from(&mut bytes).unwrap(), "");
/// ```

pub fn scan_token_from(reader: &mut Read) -> io::Result<String> {
    let mut bytes: Vec<u8> = Vec::new();

    // skip leading whitespace
    loop {
        match read_byte(reader) {
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
        match read_byte(reader) {
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
        Ok(s) => s,
        Err(err) => return Err(io::Error::new(io::ErrorKind::InvalidData, err)),
    };
    Ok(s)
}

/// reads a whitespace-delimited integer value from stdin.
///
/// # Errors
///
/// Returns error at EOF or if otherwise unable to read an integer value.

pub fn scan_int() -> io::Result<Int> {
    let stdin = io::stdin();
    let mut reader = stdin.lock();
    scan_int_from(&mut reader)
}

/// reads a whitespace-delimited integer value from a reader.
///
/// # Errors
///
/// Returns error at EOF or if otherwise unable to read an integer value.
///
/// # Example
/// ```
/// use tscp::scan::scan_int_from;
///
/// let input = String::from("  123  456 789  ");
/// let mut bytes = input.as_bytes();
/// assert_eq!(scan_int_from(&mut bytes).unwrap(), 123);
/// assert_eq!(scan_int_from(&mut bytes).unwrap(), 456);
/// assert_eq!(scan_int_from(&mut bytes).unwrap(), 789);
/// ```

pub fn scan_int_from(reader: &mut Read) -> io::Result<Int> {
    let token = scan_token_from(reader)?;
    let value: Int = match token.parse() {
        Ok(n) => n,
        Err(err) => return Err(io::Error::new(io::ErrorKind::InvalidData, err)),
    };
    Ok(value)
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
///
/// # Examples
/// ```
/// use tscp::scan::is_whitespace;
///
/// assert!(is_whitespace(' ' as u8));
/// assert!(is_whitespace('\t' as u8));
/// assert!(is_whitespace('\n' as u8));
/// assert!(is_whitespace('\r' as u8));
/// assert!(is_whitespace('\x0b' as u8));
///
/// assert!(!is_whitespace('\\' as u8));
/// assert!(!is_whitespace('\0' as u8));
/// assert!(!is_whitespace('.' as u8));
/// assert!(!is_whitespace(',' as u8));
/// ```

pub fn is_whitespace(ascii: u8) -> bool {
    match ascii {
        0x9 | 0xa | 0xb | 0xd | 0x20 => true,
        _ => false,
    }
}
