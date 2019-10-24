extern crate clap;

use clap::{App, Arg, ArgMatches};

use embedded_hal::{
    blocking::delay::DelayUs,
    digital::v2::{InputPin, OutputPin},
};
use rppal::gpio::{Gpio, IoPin, Mode};
use spin_sleep;
use std::{thread, time};
use void;

use tmledkey_hal_drv as tm;

/**
 * Raspberry pi does not have open drain pins so we have to emulate it.
 */
struct OpenPin {
    iopin: IoPin,
    mode: Mode,
}

impl OpenPin {
    fn new(mut pin: IoPin) -> OpenPin {
        pin.set_mode(Mode::Input);
        OpenPin {
            iopin: pin,
            mode: Mode::Input,
        }
    }

    fn switch_input(&mut self) {
        if self.mode != Mode::Input {
            self.mode = Mode::Input;
            self.iopin.set_mode(Mode::Input);
        }
    }

    fn switch_output(&mut self) {
        if self.mode != Mode::Output {
            self.mode = Mode::Output;
            self.iopin.set_mode(Mode::Output);
        }
    }
}

// Current rppal implementation does not support embedded_hal::gpio::v2 pins API.
impl InputPin for OpenPin {
    type Error = void::Void;

    fn is_high(&self) -> Result<bool, Self::Error> {
        Ok(self.iopin.is_high())
    }

    /// Is the input pin low?
    fn is_low(&self) -> Result<bool, Self::Error> {
        Ok(self.iopin.is_low())
    }
}

// Current rppal implementation does not support embedded_hal::gpio::v2 pins API.
impl OutputPin for OpenPin {
    type Error = void::Void;

    fn set_low(&mut self) -> Result<(), Self::Error> {
        self.switch_output();
        self.iopin.set_low();
        Ok(())
    }

    fn set_high(&mut self) -> Result<(), Self::Error> {
        self.iopin.set_high();
        self.switch_input();
        Ok(())
    }
}

struct OutPin {
    pin: rppal::gpio::OutputPin,
}
// Current rppal implementation does not support embedded_hal::gpio::v2 pins API.
impl OutputPin for OutPin {
    type Error = void::Void;

    fn set_low(&mut self) -> Result<(), Self::Error> {
        self.pin.set_low();
        Ok(())
    }

    fn set_high(&mut self) -> Result<(), Self::Error> {
        self.pin.set_high();
        Ok(())
    }
}

fn cli_matches() -> ArgMatches<'static> {
    App::new("DHT tester")
        .author("Rumato Estorsky")
        .about("TM 163xx tests")
        .arg(
            Arg::with_name("clk")
                .long("clk")
                .value_name("PIN")
                .help("CLK pin number")
                .required(true),
        )
        .arg(
            Arg::with_name("dio")
                .long("dio")
                .value_name("PIN")
                .help("DIO pin number")
                .required(true),
        )
        .get_matches()
}

struct Delayer;

impl DelayUs<u16> for Delayer {
    fn delay_us(&mut self, us: u16) {
        spin_sleep::sleep(time::Duration::from_micros(us as u64));
        //println!("D {}", us);
    }
}

fn main() {
    let matches = cli_matches();

    // println!("MATCHES {:?}", matches);

    let clk_num = matches
        .value_of("clk")
        .expect("Wrong CLK input")
        .parse::<u8>()
        .unwrap();
    let dio_num = matches
        .value_of("dio")
        .expect("Wrong DIO input")
        .parse::<u8>()
        .unwrap();

    println!("Initialized using CLK:{} DIO:{}", clk_num, dio_num);

    let gpio = Gpio::new().expect("Can not init Gpio structure");

    let clk = gpio
        .get(clk_num)
        .expect("Was not able to get CLK pin")
        .into_output();

    let dio = gpio
        .get(dio_num)
        .expect("Was not able to get CLK pin")
        .into_io(Mode::Input);

    let mut diopin = OpenPin::new(dio);
    let mut clkpin = OutPin { pin: clk };
    let mut delay = Delayer {};

    let mut delayer = || spin_sleep::sleep(time::Duration::from_micros(tm::BUS_DELAY_US_FAST as u64));

    println!("Display starts");

    let r = tm::tm_send_bytes(&mut diopin, &mut clkpin, &mut delayer, &[tm::COM_DATA_ADDRESS_ADD]);
    println!("Inited {:?}", r);

    let r = tm::tm_send_bytes(
        &mut diopin,
        &mut clkpin,
        &mut delayer,
        &[tm::COM_DISPLAY_ON | 7],
    );
    println!("Birght {:?}", r);

    let mut nums: [u8; 5] = [tm::COM_ADDRESS | 0, 1, 2, 3, 4];
    loop {
        let mut bts: [u8; 5] = [0; 5];
        bts[0] = nums[0];
        for i in 1..nums.len() {
            bts[i] = tm::CHARS[(nums[i] as usize % tm::CHARS.len())];
        }

        let r = tm::tm_send_bytes(&mut diopin, &mut clkpin, &mut delayer, &bts);
        println!("Printed {:?}", r);

        thread::sleep(time::Duration::from_millis(750));

        for i in 1..nums.len() {
            nums[i] = nums[i] + 1;
        }
    }
}