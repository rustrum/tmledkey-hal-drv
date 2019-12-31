//! Helpful functions to work with MCUs raw data.
//!
//! Some utility functions are depended on `Vec` which is require you
//! to have properly configure global memory allocator.
//!
//! If you are developing for embedded devices
//! than you probably does not have global memory allocator by default.
#[cfg(feature = "galloc")]
use alloc::vec::Vec;
use core::ops::Deref;

use super::{CHAR_MINUS, DIGITS, SEG_8};

const INT_CONVERT_MAX_SIZE: usize = 11;

/// Represents conversion result from integer to 8 segment bytemask array.
/// You could deref this structure as slice.
#[derive(Debug)]
pub struct IntConvertResult {
    offset: usize,
    bytes: [u8; INT_CONVERT_MAX_SIZE],
}

impl IntConvertResult {
    fn new() -> IntConvertResult {
        IntConvertResult {
            offset: INT_CONVERT_MAX_SIZE,
            bytes: [0; INT_CONVERT_MAX_SIZE],
        }
    }

    fn last(&self) -> u8 {
        self.bytes[self.bytes.len() - 1]
    }

    fn set_last(&mut self, byte: u8) {
        self.bytes[self.bytes.len() - 1] = byte;
    }

    fn add_first(&mut self, byte: u8) {
        if self.offset == 0 {
            return;
        }

        self.offset -= 1;
        self.bytes[self.offset] = byte;
    }

    fn add_last(&mut self, byte: u8) {
        if self.offset == 0 {
            return;
        }

        self.offset -= 1;

        for i in self.offset..(self.bytes.len() - 1) {
            self.bytes[i] = self.bytes[i + 1];
        }
        self.bytes[self.bytes.len() - 1] = byte;
    }

    fn remove_last(&mut self) {
        if self.is_empty() {
            return;
        }

        if self.len() > 1 {
            for i in ((self.offset + 1)..self.bytes.len()).rev() {
                self.bytes[i] = self.bytes[i - 1];
            }
        }
        self.offset += 1;
    }

    fn len(&self) -> usize {
        let len = self.bytes.len() - self.offset;
        if len > 0 {
            len
        } else {
            0
        }
    }

    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl Deref for IntConvertResult {
    type Target = [u8];
    fn deref(&self) -> &Self::Target {
        &self.bytes[self.offset..self.bytes.len()]
    }
}

/// Represents conversion result from float/double to 8 segment bytemask array.
/// You could deref this structure as slice.
#[derive(Debug)]
pub struct DoubleConvertResult {
    offset: usize,
    bytes: [u8; INT_CONVERT_MAX_SIZE * 2],
}

impl DoubleConvertResult {
    fn new(head: &[u8], tail: &[u8]) -> DoubleConvertResult {
        let mut offset = INT_CONVERT_MAX_SIZE * 2;
        let mut bytes = [0; INT_CONVERT_MAX_SIZE * 2];
        let len = head.len() + tail.len();
        if len <= INT_CONVERT_MAX_SIZE * 2 {
            offset -= len;
            let mut idx = offset;
            for i in 0..head.len() {
                bytes[idx] = head[i];
                idx += 1;
            }
            for i in 0..tail.len() {
                bytes[idx] = tail[i];
                idx += 1;
            }
        }

        DoubleConvertResult {
            offset: offset,
            bytes: bytes,
        }
    }
}

impl Deref for DoubleConvertResult {
    type Target = [u8];
    fn deref(&self) -> &Self::Target {
        &self.bytes[self.offset..self.bytes.len()]
    }
}

/// Convert given integer value to appropriate bytes vector.
/// Adds minus sign for negative values
pub fn int_to_bytes(value: i32) -> IntConvertResult {
    let mut result = IntConvertResult::new();
    let mut v = if value < 0 { value * -1 } else { value };
    while v > 0 {
        result.add_first(DIGITS[(v % 10) as usize]);
        v /= 10;
    }
    if value < 0 {
        result.add_first(CHAR_MINUS);
    }
    if result.is_empty() {
        result.add_first(DIGITS[0]);
    }
    result
}

/// Convert given float to appropriate bytes vector.
/// Adds minus sign for negative values, tries to ignore float garbage.
/// Always adds dot and zero for fractional part like "1.0"
pub fn float_to_bytes(value: f32) -> DoubleConvertResult {
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
pub fn float_to_bytes_ex(value: f32, precision: u8, zero_pad: bool) -> DoubleConvertResult {
    let whole = value as i32;
    let mut wresult = int_to_bytes(whole);

    let mut fract_part = fractional_part_to_bytes(value, precision);
    if zero_pad {
        while fract_part.len() < precision as usize {
            fract_part.add_last(DIGITS[0]);
        }
    }

    let with_dot = wresult.last() | SEG_8;
    wresult.set_last(with_dot);
    DoubleConvertResult::new(&wresult, &fract_part)
}

fn fractional_part_to_bytes(value: f32, precision: u8) -> IntConvertResult {
    let mut v = if value < 0.0 { value * -1.0 } else { value };
    let whole = v as i32;
    v -= whole as f32;

    let mut all_zero = true;
    let mut zeroes = 0;

    let mut result = IntConvertResult::new();
    for _ in 0..precision {
        v *= 10.0;
        let d = v as i32 % 10;
        if d == 0 {
            zeroes += 1;
        } else {
            zeroes = 0;
            all_zero = false;
        }
        result.add_last(DIGITS[(d % 10) as usize]);
        if zeroes >= 4 {
            // Skip further processing 4 or more zeroes chain found
            // Quick hack to avoid float garbage
            for i in 1..=zeroes {
                result.remove_last();
            }
            break;
        }
    }
    if all_zero {
        let mut z = IntConvertResult::new();
        z.add_last(DIGITS[0]);
        z
    } else {
        result
    }
}

/// Duplicate amount of bytes by adding 0 byte after each input byte.
/// Can be used for 3 wire interfaces with TM1638 where 2 bytes used to write display state.
///
/// This method **require "galloc"** feature to be enabled.
#[cfg(feature = "galloc")]
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
        assert_eq!(int_to_bytes(0).as_ref(), &[DIGITS[0]]);

        assert_eq!(
            int_to_bytes(1234567890).as_ref(),
            &[
                DIGITS[1], DIGITS[2], DIGITS[3], DIGITS[4], DIGITS[5], DIGITS[6], DIGITS[7],
                DIGITS[8], DIGITS[9], DIGITS[0]
            ]
        );

        assert_eq!(
            int_to_bytes(-1234567890).as_ref(),
            &[
                CHAR_MINUS, DIGITS[1], DIGITS[2], DIGITS[3], DIGITS[4], DIGITS[5], DIGITS[6],
                DIGITS[7], DIGITS[8], DIGITS[9], DIGITS[0]
            ]
        );

        assert_eq!(
            int_to_bytes(-1234).as_ref(),
            &[CHAR_MINUS, DIGITS[1], DIGITS[2], DIGITS[3], DIGITS[4]]
        );
    }

    #[test]
    fn float_to_bytes_test() {
        assert_eq!(float_to_bytes(0.0).deref(), &[DIGITS[0] | SEG_8, DIGITS[0]]);

        assert_eq!(
            float_to_bytes(-12345.0).deref(),
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
            float_to_bytes(-5.012).deref(),
            &[
                CHAR_MINUS,
                DIGITS[5] | SEG_8,
                DIGITS[0],
                DIGITS[1],
                DIGITS[2]
            ]
        );
        assert_eq!(
            float_to_bytes(12345.0).deref(),
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
            float_to_bytes(-5.20000123).deref(),
            &[CHAR_MINUS, DIGITS[5] | SEG_8, DIGITS[2]]
        );
    }

    #[test]
    fn float_to_bytes_ex_test() {
        assert_eq!(
            float_to_bytes_ex(0.0, 2, true).deref(),
            &[DIGITS[0] | SEG_8, DIGITS[0], DIGITS[0]]
        );

        assert_eq!(
            float_to_bytes_ex(0.123, 2, true).deref(),
            &[DIGITS[0] | SEG_8, DIGITS[1], DIGITS[2]]
        );

        assert_eq!(
            float_to_bytes_ex(0.123, 5, true).deref(),
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
    #[cfg(feature = "galloc")]
    fn double_bytes_test() {
        let input: [u8; 4] = [1, 2, 3, 4];
        let check: [u8; 8] = [1, 0, 2, 0, 3, 0, 4, 0];
        let result = double_bytes(&input);
        assert_eq!(check, result.as_slice());
    }
}
