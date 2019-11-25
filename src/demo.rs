use super::fx::*;
use super::*;

use alloc::vec::Vec;

pub struct Demo {
    spin: Spinner,
    slide: Slider,
    slide_last: Vec<u8>,
    displays: usize,
    iter: usize,
    brightness: u8,
}

impl Demo {
    pub fn new(displays: u8) -> Demo {
        let d = if displays <= 1 { 1 } else { displays - 1 };
        Demo {
            spin: Spinner::new(SEG_1, true),
            slide: Slider::new(SlideType::Cycle, d, &CHARS),
            slide_last: Vec::new(),
            displays: displays as usize,
            iter: 0,
            brightness: 0,
        }
    }

    pub fn init_2wire<DIO, CLK, D>(
        &mut self,
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
        clk.set_high();
        dio.set_high();
        tm_send_bytes_2wire(dio, clk, delay_us, bus_delay_us, &[COM_DATA_ADDRESS_ADD])?;
        tm_send_bytes_2wire(dio, clk, delay_us, bus_delay_us, &[COM_DISPLAY_ON])
    }

    pub fn init_3wire<DIO, CLK, STB, D>(
        &mut self,
        dio: &mut DIO,
        clk: &mut CLK,
        stb: &mut STB,
        delay_us: &mut D,
        bus_delay_us: u16,
    ) -> Result<(), TmError>
    where
        DIO: InputPin + OutputPin,
        CLK: OutputPin,
        STB: OutputPin,
        D: FnMut(u16) -> (),
    {
        clk.set_high();
        dio.set_high();
        stb.set_high();
        tm_send_bytes_3wire(
            dio,
            clk,
            stb,
            delay_us,
            bus_delay_us,
            &[COM_DATA_ADDRESS_ADD],
        )?;
        tm_send_bytes_3wire(dio, clk, stb, delay_us, bus_delay_us, &[COM_DISPLAY_ON])
    }

    pub fn next_state(&mut self) -> Vec<u8> {
        let mut result = Vec::new();

        if self.iter % 3 == 0 {
            self.slide_last = self.slide.next().unwrap();
        }

        result.append(&mut self.slide_last.clone());
        if self.displays > 1 {
            result.push(self.spin.next().unwrap());
        }

        let off = self.iter % 4;
        for i in 0..result.len() {
            if (i + off) % 4 == 0 {
                result[i] = result[i] | SEG_8;
            }
        }

        result
    }

    pub fn next_2wire<DIO, CLK, D>(
        &mut self,
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
        let mut out = self.next_state();
        let mut bytes = Vec::new();
        bytes.push(COM_ADDRESS);
        bytes.append(&mut out);
        tm_send_bytes_2wire(dio, clk, delay_us, bus_delay_us, &bytes)?;

        self.iter += 1;

        if self.iter % 10 == 0 {
            self.brightness = (self.brightness + 1) % 8;
            tm_send_bytes_2wire(
                dio,
                clk,
                delay_us,
                bus_delay_us,
                &[COM_DISPLAY_ON | (self.brightness & DISPLAY_BRIGHTNESS_MASK)],
            );
        }

        tm_read_byte_2wire(dio, clk, delay_us, bus_delay_us)
    }

    pub fn next_3wire<DIO, CLK, STB, D>(
        &mut self,
        dio: &mut DIO,
        clk: &mut CLK,
        stb: &mut STB,
        delay_us: &mut D,
        bus_delay_us: u16,
    ) -> Result<[u8; 4], TmError>
    where
        DIO: InputPin + OutputPin,
        CLK: OutputPin,
        STB: OutputPin,
        D: FnMut(u16) -> (),
    {
        let out = self.next_state();
        let mut bytes = Vec::new();
        bytes.push(COM_ADDRESS);
        let off = self.iter % 10;
        for i in 0..out.len() {
            bytes.push(out[i]);
            bytes.push(if (i + 10 - off) % 10 == 0 {
                SEG_9 | SEG_10 | SEG_11 | SEG_12
            } else {
                0
            });
        }

        tm_send_bytes_3wire(dio, clk, stb, delay_us, bus_delay_us, &bytes);

        self.iter += 1;
        if self.iter % 10 == 0 {
            self.brightness = (self.brightness + 1) % 8;
            tm_send_bytes_3wire(
                dio,
                clk,
                stb,
                delay_us,
                bus_delay_us,
                &[COM_DISPLAY_ON | (self.brightness & DISPLAY_BRIGHTNESS_MASK)],
            );
        }
        tm_read_bytes_3wire(dio, clk, stb, delay_us, bus_delay_us, 4)
    }
}

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
