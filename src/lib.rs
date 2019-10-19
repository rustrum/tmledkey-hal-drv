//#![deny(warnings)]
#![no_std]
use embedded_hal as hal;

use hal::digital::v2::{InputPin, OutputPin};
//use hal::blocking::delay::DelayUs;

/// Data command mask
pub const TM_COM_DATA: u8 = 0b01000000;
/// Address command mask
pub const TM_COM_ADR: u8 = 0b11000000;
/// Display control command mask
pub const TM_COM_DISPLAY: u8 = 0b10000000;

/// Bottom LED
pub const TM_SEGMENT_1: u8 = 0b1;
/// Bottom left LED
pub const TM_SEGMENT_2: u8 = 0b10;
/// Top left LED
pub const TM_SEGMENT_3: u8 = 0b100;
/// Top LED
pub const TM_SEGMENT_4: u8 = 0b1000;
/// Top right LED
pub const TM_SEGMENT_5: u8 = 0b10000;
/// Bottom right LED
pub const TM_SEGMENT_6: u8 = 0b100000;
/// Middle LED
pub const TM_SEGMENT_7: u8 = 0b1000000;
/// Considering that 8th segment is a colon LED
pub const TM_SEGMENT_8: u8 = 0b10000000;

#[derive(Debug)]
pub enum TmError {
    DIO,
    DIO_UP(u8),
    DIO_LOW(u8),
    CLK,
    ACK(u8),
    SEND
}

#[inline]
fn tm_bus_dio_wait_level<DIO, D>(
    dio: &mut DIO,
    bus_delay: &mut D,
    high: bool,
    err: u8,
) -> Result<(), TmError>
where
    DIO: InputPin + OutputPin,
    D: FnMut() -> (),
{
    for _ in 0..5 {
        if high == dio.is_high().map_err(|_| TmError::DIO)? {
            return Ok(());
        }
        bus_delay();
    }
    if high {
        Err(TmError::DIO_UP(err))
    } else {
        Err(TmError::DIO_LOW(err))
    }
}

#[inline]
fn tm_bus_dio_high_ack<DIO, D>(dio: &mut DIO, bus_delay: &mut D, err: u8) -> Result<(), TmError>
where
    DIO: InputPin + OutputPin,
    D: FnMut() -> (),
{
    dio.set_high().map_err(|_| TmError::DIO)?;
    tm_bus_dio_wait_level(dio, bus_delay, true, err)
}

#[inline]
fn tm_bus_dio_low_ack<DIO, D>(dio: &mut DIO, bus_delay: &mut D, err: u8) -> Result<(), TmError>
where
    DIO: InputPin + OutputPin,
    D: FnMut() -> (),
{
    dio.set_low().map_err(|_| TmError::DIO)?;
    tm_bus_dio_wait_level(dio, bus_delay, false, err)
}

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
    clk.set_high().map_err(|_| TmError::CLK)?;
    tm_bus_dio_high_ack(dio, bus_delay, 1)?;
    // bus_delay();
    // tm_bus_dio_high_ack(dio, bus_delay, 1)?;
    bus_delay();
    tm_bus_dio_low_ack(dio, bus_delay, 2)?;
    // dio.set_low().map_err(|_| TmError::DIO)?;
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
    clk.set_low().map_err(|_| TmError::CLK)?;
    bus_delay();

    //dio.set_low().map_err(|_| TmError::DIO)?;
    tm_bus_dio_low_ack(dio, bus_delay, 90)?;
    bus_delay();

    clk.set_high().map_err(|_| TmError::CLK)?;
    tm_bus_dio_high_ack(dio, bus_delay, 95)?;
    bus_delay();

    Ok(())
}

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
    // Expecting to have delay after start command
    for _ in 0..8 {
        clk.set_low().map_err(|_| TmError::CLK)?;

        let high = byte & 0b1 != 0;
        if high {
            dio.set_high().map_err(|_| TmError::DIO)?;
        // tm_bus_dio_high_ack(dio, bus_delay, 100)?;
        } else {
            dio.set_low().map_err(|_| TmError::DIO)?;
        }
        tm_bus_dio_wait_level(dio, bus_delay, high, 42)?;

        byte = byte >> 1;
        clk.set_high().map_err(|_| TmError::CLK)?;
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
) -> Result<(), TmError>
where
    DIO: InputPin + OutputPin,
    CLK: OutputPin,
    D: FnMut() -> (),
{
    //dio.set_high().map_err(|_| TmError::DIO)?;
    tm_bus_dio_high_ack(dio, bus_delay, 50).map_err(|_| TmError::ACK(0))?;

    clk.set_low().map_err(|_| TmError::CLK)?;
    // DIO to pull up - high state
    bus_delay();
    // let ack = tm_bus_dio_wait_level(dio, bus_delay, false, 50).is_ok();
    // Ensure that high DIO was pulled down
    tm_bus_dio_wait_level(dio, bus_delay, false, 51).map_err(|_| TmError::ACK(1))?;

    // 9th cycle start
    clk.set_high().map_err(|_| TmError::CLK)?;
    bus_delay();

    // Ensure pin low at 9th cycle
    tm_bus_dio_wait_level(dio, bus_delay, false, 52).map_err(|_| TmError::ACK(2))?;

    clk.set_low().map_err(|_| TmError::CLK)?;
    bus_delay();

    // Ensure pin was released and now it is up
    tm_bus_dio_wait_level(dio, bus_delay, true, 53).map_err(|_| TmError::ACK(3))?;

    Ok(())
    // if ack {
    //     Ok(())
    // } else {
    //     Err(TmError::ACK)
    // }
}

#[inline]
pub fn tm_bus_send_byte<DIO, CLK, D>(
    dio: &mut DIO,
    clk: &mut CLK,
    bus_delay: &mut D,
    byte: u8,
) -> Result<(), TmError>
where
    DIO: InputPin + OutputPin,
    CLK: OutputPin,
    D: FnMut() -> (),
{
    tm_bus_send(dio, clk, bus_delay, byte)?;
    tm_bus_ack(dio, clk, bus_delay)
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

    let mut send = Err(TmError::SEND);
    for bt in bytes {
        send = tm_bus_send_byte(dio, clk, bus_delay, bt.clone());
        if send.is_err() {
            break;
        }
    }

    let stop = tm_bus_stop(dio, clk, bus_delay);
    if send.is_err() {
        send
    } else {
        stop
    }
}
