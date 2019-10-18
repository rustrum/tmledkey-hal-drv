//#![deny(warnings)]
#![no_std]
pub mod proto;

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
    CLK,
    ACK,
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
    dio.set_high().map_err(|_| TmError::DIO)?;
    clk.set_high().map_err(|_| TmError::CLK)?;
    bus_delay();
    dio.set_low().map_err(|_| TmError::DIO)?;
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
    dio.set_low().map_err(|_| TmError::DIO)?;
    bus_delay();
    clk.set_high().map_err(|_| TmError::CLK)?;
    bus_delay();
    dio.set_high().map_err(|_| TmError::DIO)?;
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
    for _ in 0..8 {
        clk.set_low().map_err(|_| TmError::CLK)?;

        if byte & 0b1 != 0 {
            dio.set_high().map_err(|_| TmError::DIO)?;
        } else {
            dio.set_low().map_err(|_| TmError::DIO)?;
        }
        bus_delay();

        byte = byte >> 1;
        clk.set_high().map_err(|_| TmError::CLK)?;
        bus_delay();
    }
    Ok(())
}

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
    clk.set_low().map_err(|_| TmError::CLK)?;
    // DIO to pull up - high state
    dio.set_high().map_err(|_| TmError::DIO)?;
    bus_delay();
    // Second delay just in a case
    bus_delay();
    let ack = dio.is_low().map_err(|_| TmError::DIO)?;

    clk.set_high().map_err(|_| TmError::CLK)?;
    bus_delay();
    clk.set_low().map_err(|_| TmError::CLK)?;

    if ack {
        Ok(())
    } else {
        Err(TmError::ACK)
    }
}

///
/// Send command to microchip using start, send, ack and stop bus sequences.
/// 
#[inline]
pub fn tm_send_command<DIO, CLK, D>(
    dio: &mut DIO,
    clk: &mut CLK,
    bus_delay: &mut D,
    command: u8,
) -> Result<(), TmError>
where
    DIO: InputPin + OutputPin,
    CLK: OutputPin,
    D: FnMut() -> (),
{
    tm_bus_start(dio, clk, bus_delay)?;
    tm_bus_send(dio, clk, bus_delay, command)?;
    let ack = tm_bus_ack(dio, clk, bus_delay);
    // Want to send stop sequence anyway
    let stop = tm_bus_stop(dio, clk, bus_delay);
    if stop.is_err() {
        stop
    } else {
        ack
    }
}
