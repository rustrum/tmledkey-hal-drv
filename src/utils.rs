//! Utility functions are depended on `Vec` which is require you
//! to have properly configure global memory allocator.
//!
//! If you are developing for embedded devices
//! than you probably does not have global memory allocator by default.
use alloc::vec::Vec;

use super::{CHAR_MINUS, DIGITS, SEG_8};

/// Convert given integer value to appropriate bytes vector.
/// Adds minus sign for negative values
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

/// Convert given float to appropriate bytes vector.
/// Adds minus sign for negative values, tries to ignore float garbage.
/// Always adds dot and zero for fractional part like "1.0"
pub fn float_to_bytes(value: f32) -> Vec<u8> {
    float_to_bytes_ex(value, 10, false)
}

/// Extended float to bytes convertor.
/// Adds minus sign for negative values, tries to ignore float garbage.
/// Always adds dot and zero for fractional part like "1.0"
///
/// Arguments:
///  - `value` - any positive or negative value
///  - `precision` - cut off fractional part above this value
///  - `zero_pad` - pads fractional part with zeros according to `precision` value
pub fn float_to_bytes_ex(value: f32, precision: u8, zero_pad: bool) -> Vec<u8> {
    let whole = value as i32;
    let mut result = int_to_bytes(whole);
    let last_int = result.len() - 1;

    let mut fract_part = fractional_part_to_bytes(value, precision);
    if zero_pad {
        while fract_part.len() < precision as usize {
            fract_part.push(DIGITS[0]);
        }
    }

    result[last_int] = result[last_int] | SEG_8;
    result.append(&mut fract_part);
    result
}

fn fractional_part_to_bytes(value: f32, precision: u8) -> Vec<u8> {
    let mut v = if value < 0.0 { value * -1.0 } else { value };
    let whole = v as i32;
    v -= whole as f32;

    let mut all_zero = true;
    let mut zeroes = 0;
    let mut chars = Vec::<u8>::with_capacity(11);
    for _ in 0..precision {
        v *= 10.0;
        let d = v as i32 % 10;
        if d == 0 {
            zeroes += 1;
        } else {
            zeroes = 0;
            all_zero = false;
        }
        chars.push(DIGITS[(d % 10) as usize]);
        if zeroes >= 4 {
            // Skip further processing 4 or more zeroes chain found
            // Quick hack to avoid float garbage
            for i in 1..=zeroes {
                chars.pop();
            }
            break;
        }
    }
    if all_zero {
        let mut z = Vec::<u8>::with_capacity(1);
        z.push(DIGITS[0]);
        z
    } else {
        chars
    }
}

/// Duplicate amount of bytes by adding 0 byte after each input byte.
/// Can be used for 3 wire interfaces with TM1638 where 2 bytes used to write display state.
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
    fn float_to_bytes_test() {
        assert_eq!(
            float_to_bytes(0.0).as_slice(),
            &[DIGITS[0] | SEG_8, DIGITS[0]]
        );
        assert_eq!(
            float_to_bytes(12345.0).as_slice(),
            &[
                DIGITS[1],
                DIGITS[2],
                DIGITS[3],
                DIGITS[4],
                DIGITS[5] | SEG_8,
                DIGITS[0]
            ]
        );
        assert_eq!(
            float_to_bytes(-12345.0).as_slice(),
            &[
                CHAR_MINUS,
                DIGITS[1],
                DIGITS[2],
                DIGITS[3],
                DIGITS[4],
                DIGITS[5] | SEG_8,
                DIGITS[0]
            ]
        );

        assert_eq!(
            float_to_bytes(-5.012).as_slice(),
            &[
                CHAR_MINUS,
                DIGITS[5] | SEG_8,
                DIGITS[0],
                DIGITS[1],
                DIGITS[2]
            ]
        );

        assert_eq!(
            float_to_bytes(-5.20000123).as_slice(),
            &[CHAR_MINUS, DIGITS[5] | SEG_8, DIGITS[2]]
        );
    }

    #[test]
    fn float_to_bytes_ex_test() {
        assert_eq!(
            float_to_bytes_ex(0.0, 2, true).as_slice(),
            &[DIGITS[0] | SEG_8, DIGITS[0], DIGITS[0]]
        );

        assert_eq!(
            float_to_bytes_ex(0.123, 2, true).as_slice(),
            &[DIGITS[0] | SEG_8, DIGITS[1], DIGITS[2]]
        );

        assert_eq!(
            float_to_bytes_ex(0.123, 5, true).as_slice(),
            &[
                DIGITS[0] | SEG_8,
                DIGITS[1],
                DIGITS[2],
                DIGITS[3],
                DIGITS[0],
                DIGITS[0]
            ]
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
