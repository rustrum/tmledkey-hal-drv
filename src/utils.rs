//! Utility functions are depended on `Vec` which is require you
//! to have properly configure global memory allocator.
//!
//! If you are developing for embedded devices
//! than you probably does not have global memory allocator by default.
use alloc::vec::Vec;

use super::{CHAR_MINUS, DIGITS};

/// Convert given integer value to appropriate bytes vector.
pub fn int_to_bytes(value: i32) -> Vec<u8> {
    let mut chars = Vec::<u8>::with_capacity(11);
    let mut v = if value < 0 { value * -1 } else { value };
    while v > 0 {
        chars.push(DIGITS[(v % 10) as usize]);
        v /= 10;
    }
    if value < 0 {
        chars.push(CHAR_MINUS);
    }
    if chars.is_empty() {
        chars.push(DIGITS[0]);
    }
    chars.reverse();
    chars
}

/// Duplicate amount of bytes by adding 0 byte after each input byte.
/// Required for 3 wire interfaces (TM1638) where 2 bytes used to write display state.
pub fn double_bytes(input: &[u8]) -> Vec<u8> {
    let mut double_byte = Vec::<u8>::new();
    for i in 0..input.len() {
        double_byte.push(input[i]);
        double_byte.push(0);
    }
    double_byte
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn int_to_bytes_output() {
        assert_eq!(int_to_bytes(0).as_slice(), &[DIGITS[0]]);

        assert_eq!(
            int_to_bytes(1234567890).as_slice(),
            &[
                DIGITS[1], DIGITS[2], DIGITS[3], DIGITS[4], DIGITS[5], DIGITS[6], DIGITS[7],
                DIGITS[8], DIGITS[9], DIGITS[0]
            ]
        );

        assert_eq!(
            int_to_bytes(-1234567890).as_slice(),
            &[
                CHAR_MINUS, DIGITS[1], DIGITS[2], DIGITS[3], DIGITS[4], DIGITS[5], DIGITS[6],
                DIGITS[7], DIGITS[8], DIGITS[9], DIGITS[0]
            ]
        );

        assert_eq!(
            int_to_bytes(-1234).as_slice(),
            &[CHAR_MINUS, DIGITS[1], DIGITS[2], DIGITS[3], DIGITS[4]]
        );
    }

    #[test]
    fn double_bytes_test() {
        let input: [u8; 4] = [1, 2, 3, 4];
        let check: [u8; 8] = [1, 0, 2, 0, 3, 0, 4, 0];
        let result = double_bytes(&input);
        assert_eq!(check, result.as_slice());
    }
}
