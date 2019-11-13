//#![deny(warnings)]
//!
//! More infromation about [7 segment displays](https://en.wikipedia.org/wiki/Seven-SEG_display)
//!
//!
#![no_std]
#![allow(non_upper_case_globals)]
use embedded_hal as hal;

use hal::digital::v2::{InputPin, OutputPin};
//use hal::blocking::delay::DelayUs;

pub mod utils;

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

#[inline]
fn tm_bus_dio_wait_ack<DIO, D>(
    dio: &mut DIO,
    bus_delay: &mut D,
    expect_high: bool,
    err: u8,
) -> Result<(), TmError>
where
    DIO: InputPin + OutputPin,
    D: FnMut() -> (),
{
    for _ in 0..5 {
        if expect_high == dio.is_high().map_err(|_| TmError::Dio)? {
            return Ok(());
        }
        bus_delay();
    }

    Err(TmError::Ack(err))
}

/// Expecting to have initial state on bus CLK is UP and DIO is UP.
#[inline]
fn tm_bus_start<DIO, CLK, D>(dio: &mut DIO, clk: &mut CLK, bus_delay: &mut D) -> Result<(), TmError>
where
    DIO: InputPin + OutputPin,
    CLK: OutputPin,
    D: FnMut() -> (),
{
    dio.set_low().map_err(|_| TmError::Dio)?;
    bus_delay();
    Ok(())
}

#[inline]
fn tm_bus_stop<DIO, CLK, D>(dio: &mut DIO, clk: &mut CLK, bus_delay: &mut D) -> Result<(), TmError>
where
    DIO: InputPin + OutputPin,
    CLK: OutputPin,
    D: FnMut() -> (),
{
    dio.set_low().map_err(|_| TmError::Dio)?;
    bus_delay();

    clk.set_high().map_err(|_| TmError::Clk)?;
    bus_delay();

    dio.set_high().map_err(|_| TmError::Dio)?;
    tm_bus_dio_wait_ack(dio, bus_delay, true, 255)?;
    bus_delay();
    Ok(())
}

/// Expecting to have delay after start sequence or previous send call
#[inline]
fn tm_bus_send<DIO, CLK, D>(
    dio: &mut DIO,
    clk: &mut CLK,
    bus_delay: &mut D,
    mut byte: u8,
) -> Result<(), TmError>
where
    DIO: InputPin + OutputPin,
    CLK: OutputPin,
    D: FnMut() -> (),
{
    for _ in 0..8 {
        clk.set_low().map_err(|_| TmError::Clk)?;
        // This delay can be skipped, but data transfer become unstable
        bus_delay();

        let high = byte & 0b1 != 0;
        byte = byte >> 1;
        if high {
            dio.set_high().map_err(|_| TmError::Dio)?;
        } else {
            dio.set_low().map_err(|_| TmError::Dio)?;
        }
        bus_delay();

        clk.set_high().map_err(|_| TmError::Clk)?;
        bus_delay();
    }
    Ok(())
}

/// Expecting to have delay after start sequence or previous send call
#[inline]
fn tm_bus_read<DIO, CLK, D>(dio: &mut DIO, clk: &mut CLK, bus_delay: &mut D) -> Result<u8, TmError>
where
    DIO: InputPin + OutputPin,
    CLK: OutputPin,
    D: FnMut() -> (),
{
    let mut byte = 0;
    for i in 0..8 {
        clk.set_low().map_err(|_| TmError::Clk)?;
        bus_delay();
        // Looks like MCU changes dio at low CLK
        clk.set_high().map_err(|_| TmError::Clk)?;
        if dio.is_high().map_err(|_| TmError::Dio)? {
            byte = byte | 0x80 >> i;
        }
        bus_delay();
    }
    Ok(byte)
}

/// Should be called right after send
#[inline]
fn tm_bus_ack<DIO, CLK, D>(
    dio: &mut DIO,
    clk: &mut CLK,
    bus_delay: &mut D,
    err_code: u8,
    verify_last: bool,
) -> Result<(), TmError>
where
    DIO: InputPin + OutputPin,
    CLK: OutputPin,
    D: FnMut() -> (),
{
    // 8th cycle falling edge
    dio.set_high().map_err(|_| TmError::Dio)?;
    clk.set_low().map_err(|_| TmError::Clk)?;
    bus_delay();

    // Ensure that DIO was pulled down at 8th cycle falling edge
    tm_bus_dio_wait_ack(dio, bus_delay, false, err_code + 1)?;

    // 9th cycle rising edge
    clk.set_high().map_err(|_| TmError::Clk)?;
    bus_delay();

    // Ensure DIO still low at 9th cycle rising edge
    tm_bus_dio_wait_ack(dio, bus_delay, false, err_code + 2)?;

    // 9th cycle falling edge
    clk.set_low().map_err(|_| TmError::Clk)?;
    bus_delay();

    // Ensure DIO was released and now it is up
    if verify_last {
        // No need to check last ack for reading mode
        tm_bus_dio_wait_ack(dio, bus_delay, true, err_code + 3)?;
    }

    Ok(())
}

#[inline]
fn tm_bus_send_byte_ack<DIO, CLK, D>(
    dio: &mut DIO,
    clk: &mut CLK,
    bus_delay: &mut D,
    byte: u8,
    err_code: u8,
) -> Result<(), TmError>
where
    DIO: InputPin + OutputPin,
    CLK: OutputPin,
    D: FnMut() -> (),
{
    tm_bus_send(dio, clk, bus_delay, byte)?;
    let verify_last = byte & COM_DATA_READ != COM_DATA_READ;
    tm_bus_ack(dio, clk, bus_delay, err_code, verify_last)
}

#[inline]
fn tm_bus_read_byte<DIO, CLK, D>(
    dio: &mut DIO,
    clk: &mut CLK,
    bus_delay: &mut D,
    err_code: u8,
) -> Result<u8, TmError>
where
    DIO: InputPin + OutputPin,
    CLK: OutputPin,
    D: FnMut() -> (),
{
    let result = tm_bus_read(dio, clk, bus_delay);
    tm_bus_ack(dio, clk, bus_delay, err_code, true)?;
    result
}

///
/// Send one or several bytes to MCU via 2 wire interface (DIO,CLK).
/// Accoding to datasheet it can be single commad byte or a sequence starting with command byte followed by several data bytes.
///
#[inline]
pub fn tm_send_bytes_2wire<DIO, CLK, D>(
    dio: &mut DIO,
    clk: &mut CLK,
    bus_delay: &mut D,
    bytes: &[u8],
) -> Result<(), TmError>
where
    DIO: InputPin + OutputPin,
    CLK: OutputPin,
    D: FnMut() -> (),
{
    tm_bus_start(dio, clk, bus_delay)?;

    let mut send = Err(TmError::Input);
    let mut iter = 10;
    for bt in bytes {
        send = tm_bus_send_byte_ack(dio, clk, bus_delay, bt.clone(), iter);
        if send.is_err() {
            break;
        }
        iter += 10;
    }

    let stop = tm_bus_stop(dio, clk, bus_delay);
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
pub fn tm_read_byte_2wire<DIO, CLK, D>(
    dio: &mut DIO,
    clk: &mut CLK,
    bus_delay: &mut D,
) -> Result<u8, TmError>
where
    DIO: InputPin + OutputPin,
    CLK: OutputPin,
    D: FnMut() -> (),
{
    tm_bus_start(dio, clk, bus_delay)?;

    tm_bus_send_byte_ack(dio, clk, bus_delay, COM_DATA_READ, 230)?;

    let read = tm_bus_read_byte(dio, clk, bus_delay, 240);

    let stop = tm_bus_stop(dio, clk, bus_delay);
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
    stb.set_low().map_err(|_| TmError::Stb)?;
    delay_us(bus_delay_us);

    let mut send = Err(TmError::Input);
    let mut delayer = || delay_us(bus_delay_us);
    for bt in bytes {
        send = tm_bus_send(dio, clk, &mut delayer, bt.clone());
        if send.is_err() {
            break;
        }
        // Notice: When read data, set instruction from the 8th rising edge of clock
        // to CLK falling edge to read data that demand a waiting time Twait(min 1μS).
        //delayer();
    }
    delay_us(bus_delay_us);
    stb.set_high().map_err(|_| TmError::Stb)?;
    send
}

///
/// Read *length* of bytes from MCU using 3 wire interface (DIO,CLK,STB).
///
#[inline]
pub fn tm_read_bytes_3wire<'a, DIO, CLK, STB, D>(
    dio: &mut DIO,
    clk: &mut CLK,
    stb: &mut STB,
    delay_us: &mut D,
    bus_delay_us: u16,
    length: u8,
) -> Result<&'a [u8], TmError>
where
    DIO: InputPin + OutputPin,
    CLK: OutputPin,
    STB: OutputPin,
    D: FnMut(u16) -> (),
{
    Ok(&[0])
}

/// Number of bytes that can be read from from TM1638 response.
pub const TM1638_RESPONSE_SIZE: u8 = 4;
/// Maximum number of display segments supported by this MCU.
pub const TM1638_MAX_SEGMENTS: u8 = 10;

/// Number of bytes that can be read from from TM1637 response.
pub const TM1637_RESPONSE_SIZE: u8 = 1;
/// Maximum number of display segments supported by this MCU.
pub const TM1637_MAX_SEGMENTS: u8 = 6;

/// Resonable delay for TM serial protocol.
///
/// This value should fit all configurations, but you can adjust it.
/// Delay value may vary depending on your circuit (consider pull up resitors).
pub const BUS_DELAY_US: u16 = 500;

/// Lower but sill working delay accroding to my own tests.
pub const BUS_DELAY_US_FAST: u16 = 350;

/// Address adding mode (write to display)
pub const COM_DATA_ADDRESS_ADD: u8 = 0b01000000;
/// Data fix address mode (write to display)
pub const COM_DATA_ADDRESS_FIXED: u8 = 0b01000100;
/// Read key scan data
pub const COM_DATA_READ: u8 = 0b01000010;

/// Register address command mask
pub const COM_ADDRESS: u8 = 0b11000000;

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

/// List of all available characters including numbers
pub const CHARS: [u8; 47] = [
    CHAR_0,
    CHAR_1,
    CHAR_2,
    CHAR_3,
    CHAR_4,
    CHAR_5,
    CHAR_6,
    CHAR_7,
    CHAR_8,
    CHAR_9,
    CHAR_A,
    CHAR_a,
    CHAR_b,
    CHAR_C,
    CHAR_c,
    CHAR_d,
    CHAR_E,
    CHAR_e,
    CHAR_F,
    CHAR_G,
    CHAR_H,
    CHAR_h,
    CHAR_I,
    CHAR_i,
    CHAR_J,
    CHAR_L,
    CHAR_l,
    CHAR_N,
    CHAR_n,
    CHAR_O,
    CHAR_o,
    CHAR_P,
    CHAR_q,
    CHAR_R,
    CHAR_r,
    CHAR_S,
    CHAR_t,
    CHAR_U,
    CHAR_u,
    CHAR_y,
    CHAR_CYR_E,
    CHAR_CYR_B,
    CHAR_DEGREE,
    CHAR_MINUS,
    CHAR_UNDERSCORE,
    CHAR_BRACKET_LEFT,
    CHAR_BRACKET_RIGHT,
];
