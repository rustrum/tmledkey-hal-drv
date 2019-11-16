use alloc::vec::Vec;

use super::{CHAR_MINUS, DIGITS};

///
/// Convert given integer value to appropriate bytes vector.
/// Require to have properly configured memory allocator in your project.
///
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
}
