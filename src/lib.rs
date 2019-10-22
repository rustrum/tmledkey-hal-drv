//#![deny(warnings)]
//!
//! More infromation about [7 segment displays](https://en.wikipedia.org/wiki/Seven-segment_display)
//!
//!
#![no_std]
use embedded_hal as hal;

use hal::digital::v2::{InputPin, OutputPin};
//use hal::blocking::delay::DelayUs;

/// Address adding mode (write to display)
pub const TM_COM_DATA_ADDR_ADDING: u8 = 0b01000000;
/// Data fix address mode (write to display)
pub const TM_COM_DATA_ADDR_FIX: u8 = 0b01000100;
/// Read key scan data
pub const TM_COM_DATA_READ: u8 = 0b01000010;

/// Register address command mask
pub const TM_COM_ADR: u8 = 0b11000000;

/// Display ON max brightness.
/// Can be combined with masked bytes to adjust brightness level
pub const TM_COM_DISPLAY_ON: u8 = 0b10001000;
/// Display brightness mask
pub const TM_DISPLAY_BRIGHTNESS_MASK: u8 = 0b00000111;
// Display OFF
pub const TM_COM_DISPLAY_OFF: u8 = 0b10000000;

/// Segment A - top
pub const TM_SEGMENT_1: u8 = 0b1;
/// Segment B - top right
pub const TM_SEGMENT_2: u8 = 0b10;
/// Segment C - bottom right
pub const TM_SEGMENT_3: u8 = 0b100;
/// Segment D - bottom
pub const TM_SEGMENT_4: u8 = 0b1000;
/// Segment E - bottom left
pub const TM_SEGMENT_5: u8 = 0b10000;
/// Segment F - top left
pub const TM_SEGMENT_6: u8 = 0b100000;
/// Segment G - middle
pub const TM_SEGMENT_7: u8 = 0b1000000;
/// Segment DP (eight) - dot or colon
pub const TM_SEGMENT_8: u8 = 0b10000000;

#[derive(Debug)]
pub enum TmError {
    Dio,
    Ack(u8),
    Clk,
    Empty,
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
    for _ in 0..10 {
        if expect_high == dio.is_high().map_err(|_| TmError::Dio)? {
            return Ok(());
        }
        bus_delay();
    }

    Err(TmError::Ack(err))
}

/// Expecting to have initial state on bus CLK is UP and DIO is UP.
#[inline]
pub fn tm_bus_start<DIO, CLK, D>(
    dio: &mut DIO,
    clk: &mut CLK,
    bus_delay: &mut D,
) -> Result<(), TmError>
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
pub fn tm_bus_stop<DIO, CLK, D>(
    dio: &mut DIO,
    clk: &mut CLK,
    bus_delay: &mut D,
) -> Result<(), TmError>
where
    DIO: InputPin + OutputPin,
    CLK: OutputPin,
    D: FnMut() -> (),
{
    // clk.set_low().map_err(|_| TmError::Clk)?;
    // bus_delay();

    dio.set_low().map_err(|_| TmError::Dio)?;
    // tm_bus_dio_low_ack(dio, bus_delay, 90)?;
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
pub fn tm_bus_send<DIO, CLK, D>(
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

/// Should be called after
#[inline]
pub fn tm_bus_ack<DIO, CLK, D>(
    dio: &mut DIO,
    clk: &mut CLK,
    bus_delay: &mut D,
    err_code: u8,
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
    tm_bus_dio_wait_ack(dio, bus_delay, true, err_code + 3)?;

    Ok(())
}

#[inline]
pub fn tm_bus_send_byte<DIO, CLK, D>(
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
    tm_bus_ack(dio, clk, bus_delay, err_code)
}

///
/// Send command to microchip using start, send, ack and stop bus sequences.
///
#[inline]
pub fn tm_send_bytes<DIO, CLK, D>(
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

    let mut send = Err(TmError::Empty);
    let mut iter = 10;
    for bt in bytes {
        send = tm_bus_send_byte(dio, clk, bus_delay, bt.clone(), iter);
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
