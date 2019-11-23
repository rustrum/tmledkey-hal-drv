//#![deny(warnings)]
//!
//! More infromation about [7 segment displays](https://en.wikipedia.org/wiki/Seven-SEG_display)
//!
//!
#![no_std]
#![allow(non_upper_case_globals)]
extern crate alloc;

#[cfg(feature = "keys")]
use alloc::vec::Vec;

#[cfg(feature = "utils")]
pub mod utils;

#[cfg(feature = "fx")]
pub mod fx;

#[cfg(feature = "demo")]
pub mod demo;

//  #[cfg(test)]
//  #[cfg(feature = "demo")]
//  #[macro_use]
// extern crate std;
// extern crate std;
// use std::alloc::System;

// #[global_allocator]
// static A: System = System;

use embedded_hal::digital::v2::{InputPin, OutputPin};

#[cfg(not(any(feature = "clkdio", feature = "clkdiostb")))]
compile_error!("Either feature \"clkdio\" or \"clkdiostb\" must be enabled for this crate. Otherwise there is no sence to use it");

#[derive(Debug)]
pub enum TmError {
    Dio,
    /// ACK error with tricky code.
    /// Code could help to get more detaled information about exact place where error occured.
    Ack(u8),
    Clk,
    Stb,
    /// There was some errors in user input
    Input,
}

/// Expecting to have delay after start sequence or previous send call
#[inline]
fn tm_bus_send<DIO, CLK, D>(
    dio: &mut DIO,
    clk: &mut CLK,
    delay_us: &mut D,
    bus_delay_us: u16,
    mut byte: u8,
) -> Result<(), TmError>
where
    DIO: InputPin + OutputPin,
    CLK: OutputPin,
    D: FnMut(u16) -> (),
{
    for _ in 0..8 {
        clk.set_low().map_err(|_| TmError::Clk)?;
        // This delay can be skipped, but data transfer become unstable
        delay_us(bus_delay_us);

        let high = byte & 0b1 != 0;
        byte = byte >> 1;
        if high {
            dio.set_high().map_err(|_| TmError::Dio)?;
        } else {
            dio.set_low().map_err(|_| TmError::Dio)?;
        }
        delay_us(bus_delay_us);

        clk.set_high().map_err(|_| TmError::Clk)?;
        delay_us(bus_delay_us);
    }
    Ok(())
}

/// Expecting to have delay after start sequence or previous send call
#[inline]
#[cfg(feature = "keys")]
fn tm_bus_read<DIO, CLK, D>(
    dio: &mut DIO,
    clk: &mut CLK,
    delay_us: &mut D,
    bus_delay_us: u16,
) -> Result<u8, TmError>
where
    DIO: InputPin + OutputPin,
    CLK: OutputPin,
    D: FnMut(u16) -> (),
{
    let mut byte = 0;
    for i in 0..8 {
        clk.set_low().map_err(|_| TmError::Clk)?;
        delay_us(bus_delay_us);
        // Looks like MCU changes dio at low CLK
        clk.set_high().map_err(|_| TmError::Clk)?;
        if dio.is_high().map_err(|_| TmError::Dio)? {
            byte = byte | 0x80 >> i;
        }
        delay_us(bus_delay_us);
    }
    Ok(byte)
}

#[inline]
#[cfg(feature = "clkdio")]
fn tm_bus_dio_wait_ack<DIO, D>(
    dio: &mut DIO,
    delay_us: &mut D,
    bus_delay_us: u16,
    expect_high: bool,
    err: u8,
) -> Result<(), TmError>
where
    DIO: InputPin + OutputPin,
    D: FnMut(u16) -> (),
{
    for _ in 0..5 {
        if expect_high == dio.is_high().map_err(|_| TmError::Dio)? {
            return Ok(());
        }
        delay_us(bus_delay_us);
    }

    Err(TmError::Ack(err))
}

/// Expecting to have initial state on bus CLK is UP and DIO is UP.
#[inline]
#[cfg(feature = "clkdio")]
fn tm_bus_2wire_start<DIO, CLK, D>(
    dio: &mut DIO,
    clk: &mut CLK,
    delay_us: &mut D,
    bus_delay_us: u16,
) -> Result<(), TmError>
where
    DIO: InputPin + OutputPin,
    CLK: OutputPin,
    D: FnMut(u16) -> (),
{
    dio.set_low().map_err(|_| TmError::Dio)?;
    delay_us(bus_delay_us);
    Ok(())
}

#[inline]
#[cfg(feature = "clkdio")]
fn tm_bus_2wire_stop<DIO, CLK, D>(
    dio: &mut DIO,
    clk: &mut CLK,
    delay_us: &mut D,
    bus_delay_us: u16,
) -> Result<(), TmError>
where
    DIO: InputPin + OutputPin,
    CLK: OutputPin,
    D: FnMut(u16) -> (),
{
    dio.set_low().map_err(|_| TmError::Dio)?;
    delay_us(bus_delay_us);

    clk.set_high().map_err(|_| TmError::Clk)?;
    delay_us(bus_delay_us);

    dio.set_high().map_err(|_| TmError::Dio)?;
    tm_bus_dio_wait_ack(dio, delay_us, bus_delay_us, true, 255)?;
    delay_us(bus_delay_us);
    Ok(())
}

/// Should be called right after send
#[inline]
#[cfg(feature = "clkdio")]
fn tm_bus_2wire_ack<DIO, CLK, D>(
    dio: &mut DIO,
    clk: &mut CLK,
    delay_us: &mut D,
    bus_delay_us: u16,
    err_code: u8,
    verify_last: bool,
) -> Result<(), TmError>
where
    DIO: InputPin + OutputPin,
    CLK: OutputPin,
    D: FnMut(u16) -> (),
{
    // 8th cycle falling edge
    dio.set_high().map_err(|_| TmError::Dio)?;
    clk.set_low().map_err(|_| TmError::Clk)?;
    delay_us(bus_delay_us);

    // Ensure that DIO was pulled down at 8th cycle falling edge
    tm_bus_dio_wait_ack(dio, delay_us, bus_delay_us, false, err_code + 1)?;

    // 9th cycle rising edge
    clk.set_high().map_err(|_| TmError::Clk)?;
    delay_us(bus_delay_us);

    // Ensure DIO still low at 9th cycle rising edge
    tm_bus_dio_wait_ack(dio, delay_us, bus_delay_us, false, err_code + 2)?;

    // 9th cycle falling edge
    clk.set_low().map_err(|_| TmError::Clk)?;
    delay_us(bus_delay_us);

    // Ensure DIO was released and now it is up
    if verify_last {
        // No need to check last ack for reading mode
        tm_bus_dio_wait_ack(dio, delay_us, bus_delay_us, true, err_code + 3)?;
    }

    Ok(())
}

#[inline]
#[cfg(feature = "clkdio")]
fn tm_bus_2wire_send_byte_ack<DIO, CLK, D>(
    dio: &mut DIO,
    clk: &mut CLK,
    delay_us: &mut D,
    bus_delay_us: u16,
    byte: u8,
    err_code: u8,
) -> Result<(), TmError>
where
    DIO: InputPin + OutputPin,
    CLK: OutputPin,
    D: FnMut(u16) -> (),
{
    tm_bus_send(dio, clk, delay_us, bus_delay_us, byte)?;
    let verify_last = byte & COM_DATA_READ != COM_DATA_READ;
    tm_bus_2wire_ack(dio, clk, delay_us, bus_delay_us, err_code, verify_last)
}

#[inline]
#[cfg(all(feature = "keys", feature = "clkdio"))]
fn tm_bus_2wire_read_byte_ack<DIO, CLK, D>(
    dio: &mut DIO,
    clk: &mut CLK,
    delay_us: &mut D,
    bus_delay_us: u16,
    err_code: u8,
) -> Result<u8, TmError>
where
    DIO: InputPin + OutputPin,
    CLK: OutputPin,
    D: FnMut(u16) -> (),
{
    let result = tm_bus_read(dio, clk, delay_us, bus_delay_us);
    tm_bus_2wire_ack(dio, clk, delay_us, bus_delay_us, err_code, true)?;
    result
}

///
/// Send one or several bytes to MCU via 2 wire interface (DIO,CLK).
/// Accoding to datasheet it can be single commad byte or a sequence starting with command byte followed by several data bytes.
///
#[inline]
#[cfg(feature = "clkdio")]
pub fn tm_send_bytes_2wire<DIO, CLK, D>(
    dio: &mut DIO,
    clk: &mut CLK,
    delay_us: &mut D,
    bus_delay_us: u16,
    bytes: &[u8],
) -> Result<(), TmError>
where
    DIO: InputPin + OutputPin,
    CLK: OutputPin,
    D: FnMut(u16) -> (),
{
    tm_bus_2wire_start(dio, clk, delay_us, bus_delay_us)?;

    let mut send = Err(TmError::Input);
    let mut iter = 10;
    for bt in bytes {
        send = tm_bus_2wire_send_byte_ack(dio, clk, delay_us, bus_delay_us, bt.clone(), iter);
        if send.is_err() {
            break;
        }
        iter += 10;
    }

    let stop = tm_bus_2wire_stop(dio, clk, delay_us, bus_delay_us);
    if send.is_err() {
        send
    } else {
        stop
    }
}

///
/// Reads key scan data as byte via 2 wire interface (DIO,CLK).
///
#[inline]
#[cfg(all(feature = "keys", feature = "clkdio"))]
pub fn tm_read_byte_2wire<DIO, CLK, D>(
    dio: &mut DIO,
    clk: &mut CLK,
    delay_us: &mut D,
    bus_delay_us: u16,
) -> Result<u8, TmError>
where
    DIO: InputPin + OutputPin,
    CLK: OutputPin,
    D: FnMut(u16) -> (),
{
    tm_bus_2wire_start(dio, clk, delay_us, bus_delay_us)?;

    tm_bus_2wire_send_byte_ack(dio, clk, delay_us, bus_delay_us, COM_DATA_READ, 230)?;

    let read = tm_bus_2wire_read_byte_ack(dio, clk, delay_us, bus_delay_us, 240);

    let stop = tm_bus_2wire_stop(dio, clk, delay_us, bus_delay_us);
    if stop.is_err() {
        if read.is_err() {
            return read;
        } else {
            return Err(stop.err().unwrap());
        }
    }
    read
}

///
/// Send bytes using 3 wire interface (DIO,CLK,STB).
///
#[inline]
#[cfg(feature = "clkdiostb")]
pub fn tm_send_bytes_3wire<DIO, CLK, STB, D>(
    dio: &mut DIO,
    clk: &mut CLK,
    stb: &mut STB,
    delay_us: &mut D,
    bus_delay_us: u16,
    bytes: &[u8],
) -> Result<(), TmError>
where
    DIO: InputPin + OutputPin,
    CLK: OutputPin,
    STB: OutputPin,
    D: FnMut(u16) -> (),
{
    delay_us(bus_delay_us);
    stb.set_low().map_err(|_| TmError::Stb)?;
    delay_us(bus_delay_us);

    let mut send = Err(TmError::Input);
    for bt in bytes {
        send = tm_bus_send(dio, clk, delay_us, bus_delay_us, bt.clone());
        if send.is_err() {
            break;
        }
        // Notice: When read data, set instruction from the 8th rising edge of clock
        // to CLK falling edge to read data that demand a waiting time Twait(min 1μS).
        //delayer();
    }
    delay_us(bus_delay_us);
    stb.set_high().map_err(|_| TmError::Stb)?;
    clk.set_high().map_err(|_| TmError::Stb)?;
    dio.set_high().map_err(|_| TmError::Stb)?;
    send
}

///
/// Read **length** of bytes from MCU using 3 wire interface (DIO,CLK,STB).
///
#[inline]
#[cfg(all(feature = "keys", feature = "clkdiostb"))]
pub fn tm_read_bytes_3wire<DIO, CLK, STB, D>(
    dio: &mut DIO,
    clk: &mut CLK,
    stb: &mut STB,
    delay_us: &mut D,
    bus_delay_us: u16,
    length: u8,
) -> Result<Vec<u8>, TmError>
where
    DIO: InputPin + OutputPin,
    CLK: OutputPin,
    STB: OutputPin,
    D: FnMut(u16) -> (),
{
    let mut read_err = None;
    let mut response = Vec::<u8>::with_capacity(length as usize);

    delay_us(bus_delay_us);
    stb.set_low().map_err(|_| TmError::Stb)?;
    delay_us(bus_delay_us);

    let res_init = tm_bus_send(dio, clk, delay_us, bus_delay_us, COM_DATA_READ);
    dio.set_high().map_err(|_| TmError::Stb)?;
    if res_init.is_err() {
        read_err = Some(res_init.unwrap_err());
    } else {
        // Notice: When read data, set instruction from the 8th rising edge of clock
        // to CLK falling edge to read data that demand a waiting time Twait(min 1μS).
        delay_us(bus_delay_us);
        for _ in 0..length {
            match tm_bus_read(dio, clk, delay_us, bus_delay_us) {
                Ok(b) => {
                    response.push(b);
                }
                Err(e) => {
                    read_err = Some(e);
                    break;
                }
            }
        }
    }

    stb.set_high().map_err(|_| TmError::Stb)?;
    clk.set_high().map_err(|_| TmError::Stb)?;
    dio.set_high().map_err(|_| TmError::Stb)?;

    if read_err.is_some() {
        return Err(read_err.unwrap());
    }
    Ok(response)
}

/// Number of bytes that can be read from from TM1638 response.
pub const TM1638_RESPONSE_SIZE: u8 = 4;
/// Maximum number of display segments supported by this MCU.
pub const TM1638_MAX_SEGMENTS: u8 = 10;

/// Number of bytes that can be read from from TM1637 response.
pub const TM1637_RESPONSE_SIZE: u8 = 1;
/// Maximum number of display segments supported by this MCU.
pub const TM1637_MAX_SEGMENTS: u8 = 6;

/// Prooven working delay for TM1637
pub const TM1637_BUS_DELAY_US: u16 = 400;

/// Prooven working delay for TM1638
pub const TM1638_BUS_DELAY_US: u16 = 1;

/// Resonable delay for TM serial protocol.
///
/// This value should fit all configurations, but you can adjust it.
/// Delay value may vary depending on your circuit (consider pull up resitors).
pub const BUS_DELAY_US: u16 = 400;

/// Data control instruction set
pub const COM_DATA: u8 = 0b01000000;

/// Display control instruction set
pub const COM_DISPLAY: u8 = 0b10000000;

/// Address instruction set
pub const COM_ADDRESS: u8 = 0b11000000;

/// Address adding mode (write to display)
pub const COM_DATA_ADDRESS_ADD: u8 = COM_DATA | 0b000000;
/// Data fix address mode (write to display)
pub const COM_DATA_ADDRESS_FIXED: u8 = COM_DATA | 0b000100;
/// Read key scan data
pub const COM_DATA_READ: u8 = COM_DATA | 0b000010;

/// Display ON max brightness.
/// Can be combined with masked bytes to adjust brightness level
pub const COM_DISPLAY_ON: u8 = 0b10001000;
/// Display brightness mask
pub const DISPLAY_BRIGHTNESS_MASK: u8 = 0b00000111;
// Display OFF
pub const COM_DISPLAY_OFF: u8 = 0b10000000;

/// Segment A - top
pub const SEG_1: u8 = 0b1;
/// Segment B - top right
pub const SEG_2: u8 = 0b10;
/// Segment C - bottom right
pub const SEG_3: u8 = 0b100;
/// Segment D - bottom
pub const SEG_4: u8 = 0b1000;
/// Segment E - bottom left
pub const SEG_5: u8 = 0b10000;
/// Segment F - top left
pub const SEG_6: u8 = 0b100000;
/// Segment G - middle
pub const SEG_7: u8 = 0b1000000;
/// Segment DP (eight) - dot or colon
pub const SEG_8: u8 = 0b10000000;

/// Used with 3 wire interface for second byte
pub const SEG_9: u8 = SEG_1;
/// Used with 3 wire interface for second byte
pub const SEG_10: u8 = SEG_2;
/// Used with 3 wire interface for second byte
pub const SEG_11: u8 = SEG_3;
/// Used with 3 wire interface for second byte
pub const SEG_12: u8 = SEG_4;

pub const CHAR_0: u8 = SEG_1 | SEG_2 | SEG_3 | SEG_4 | SEG_5 | SEG_6;
pub const CHAR_1: u8 = SEG_2 | SEG_3;
pub const CHAR_2: u8 = SEG_1 | SEG_2 | SEG_4 | SEG_5 | SEG_7;
pub const CHAR_3: u8 = SEG_1 | SEG_2 | SEG_3 | SEG_4 | SEG_7;
pub const CHAR_4: u8 = SEG_2 | SEG_3 | SEG_6 | SEG_7;
pub const CHAR_5: u8 = SEG_1 | SEG_3 | SEG_4 | SEG_6 | SEG_7;
pub const CHAR_6: u8 = SEG_1 | SEG_3 | SEG_4 | SEG_5 | SEG_6 | SEG_7;
pub const CHAR_7: u8 = SEG_1 | SEG_2 | SEG_3;
pub const CHAR_8: u8 = SEG_1 | SEG_2 | SEG_3 | SEG_4 | SEG_5 | SEG_6 | SEG_7;
pub const CHAR_9: u8 = SEG_1 | SEG_2 | SEG_3 | SEG_4 | SEG_6 | SEG_7;
pub const CHAR_A: u8 = SEG_1 | SEG_2 | SEG_3 | SEG_5 | SEG_6 | SEG_7;
pub const CHAR_a: u8 = SEG_1 | SEG_2 | SEG_3 | SEG_4 | SEG_5 | SEG_7;
pub const CHAR_b: u8 = SEG_3 | SEG_4 | SEG_5 | SEG_6 | SEG_7;
pub const CHAR_C: u8 = SEG_1 | SEG_4 | SEG_5 | SEG_6;
pub const CHAR_c: u8 = SEG_4 | SEG_5 | SEG_7;
pub const CHAR_d: u8 = SEG_2 | SEG_3 | SEG_4 | SEG_5 | SEG_7;
pub const CHAR_E: u8 = SEG_1 | SEG_4 | SEG_5 | SEG_6 | SEG_7;
pub const CHAR_e: u8 = SEG_1 | SEG_2 | SEG_4 | SEG_5 | SEG_6 | SEG_7;
pub const CHAR_F: u8 = SEG_1 | SEG_5 | SEG_6 | SEG_7;
pub const CHAR_G: u8 = SEG_1 | SEG_3 | SEG_4 | SEG_5 | SEG_6;
pub const CHAR_H: u8 = SEG_2 | SEG_3 | SEG_5 | SEG_6 | SEG_7;
pub const CHAR_h: u8 = SEG_3 | SEG_5 | SEG_6 | SEG_7;
pub const CHAR_I: u8 = SEG_2 | SEG_3;
pub const CHAR_i: u8 = SEG_3;
pub const CHAR_J: u8 = SEG_2 | SEG_3 | SEG_4 | SEG_5;
pub const CHAR_L: u8 = SEG_4 | SEG_5 | SEG_6;
pub const CHAR_l: u8 = SEG_4 | SEG_5;
pub const CHAR_N: u8 = SEG_1 | SEG_2 | SEG_3 | SEG_5 | SEG_6;
pub const CHAR_n: u8 = SEG_3 | SEG_5 | SEG_7;
pub const CHAR_O: u8 = SEG_1 | SEG_2 | SEG_3 | SEG_4 | SEG_5 | SEG_6;
pub const CHAR_o: u8 = SEG_3 | SEG_4 | SEG_5 | SEG_7;
pub const CHAR_P: u8 = SEG_1 | SEG_2 | SEG_5 | SEG_6 | SEG_7;
pub const CHAR_q: u8 = SEG_1 | SEG_2 | SEG_3 | SEG_6 | SEG_7;
pub const CHAR_R: u8 = SEG_1 | SEG_5 | SEG_6;
pub const CHAR_r: u8 = SEG_5 | SEG_7;
pub const CHAR_S: u8 = SEG_1 | SEG_3 | SEG_4 | SEG_6 | SEG_7;
pub const CHAR_t: u8 = SEG_4 | SEG_5 | SEG_6 | SEG_7;
pub const CHAR_U: u8 = SEG_2 | SEG_3 | SEG_4 | SEG_5 | SEG_6;
pub const CHAR_u: u8 = SEG_3 | SEG_4 | SEG_5;
pub const CHAR_y: u8 = SEG_2 | SEG_3 | SEG_4 | SEG_6 | SEG_7;
pub const CHAR_CYR_E: u8 = SEG_1 | SEG_2 | SEG_3 | SEG_4 | SEG_7;
pub const CHAR_CYR_B: u8 = SEG_1 | SEG_3 | SEG_4 | SEG_5 | SEG_6 | SEG_7;
pub const CHAR_DEGREE: u8 = SEG_1 | SEG_2 | SEG_6 | SEG_7;
pub const CHAR_MINUS: u8 = SEG_7;
pub const CHAR_UNDERSCORE: u8 = SEG_4;
pub const CHAR_BRACKET_LEFT: u8 = SEG_1 | SEG_4 | SEG_5 | SEG_6;
pub const CHAR_BRACKET_RIGHT: u8 = SEG_1 | SEG_2 | SEG_3 | SEG_4;

/// List of digit characters where values correlates with array index 0-9.
pub const DIGITS: [u8; 10] = [
    CHAR_0, CHAR_1, CHAR_2, CHAR_3, CHAR_4, CHAR_5, CHAR_6, CHAR_7, CHAR_8, CHAR_9,
];
