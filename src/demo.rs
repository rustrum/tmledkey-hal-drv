use super::fx::*;
use super::*;

pub struct Demo {
    spin: Spinner,
    slide: Slider,
    displays: usize,
    iter: usize,
}

impl Demo {
    pub fn new(displays: u8) -> Demo {
        let d = if displays <= 1 { 1 } else { displays - 1 };
        Demo {
            spin: Spinner::new((CHAR_0 & !SEG_1) & !SEG_4, true),
            slide: Slider::new(SlideType::Cycle, d, &CHARS),
            displays: displays as usize,
            iter: 0,
        }
    }

    pub fn next_state(&mut self) -> Vec<u8> {
        let mut result = Vec::new();

        result.append(&mut self.slide.next().unwrap());
        if self.displays > 1 {
            result.push(self.spin.next().unwrap());
        }
        result
    }

    pub fn next_2wire<DIO, CLK, STB, D>(
        &mut self,
        dio: &mut DIO,
        clk: &mut CLK,
        delay_us: &mut D,
        bus_delay_us: u16,
    ) where
        DIO: InputPin + OutputPin,
        CLK: OutputPin,
        D: FnMut(u16) -> (),
    {
        let mut out = self.next_state();
        let mut bytes = Vec::new();
        bytes.push(COM_ADDRESS);
        bytes.append(&mut out);
        tm_send_bytes_2wire(dio, clk, delay_us, bus_delay_us, &bytes);

        self.iter += 1;
    }

    pub fn next_3wire<DIO, CLK, STB, D>(
        &mut self,
        dio: &mut DIO,
        clk: &mut CLK,
        stb: &mut STB,
        delay_us: &mut D,
        bus_delay_us: u16,
    ) where
        DIO: InputPin + OutputPin,
        CLK: OutputPin,
        STB: OutputPin,
        D: FnMut(u16) -> (),
    {
        let mut out = self.next_state();
        let mut bytes = Vec::new();
        bytes.push(COM_ADDRESS);
        bytes.append(&mut out);
        tm_send_bytes_3wire(dio, clk, stb, delay_us, bus_delay_us, &bytes);

        // tm_read_bytes_3wire(dio, clk, stb, delay_us, bus_delay_us, &bytes);
        self.iter += 1;
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
