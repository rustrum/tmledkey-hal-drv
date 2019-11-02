//#![deny(unsafe_code)]
#![no_std]
#![no_main]

use panic_halt as _;

use cortex_m_rt::entry;
use cortex_m_semihosting::hprintln;
use stm32f1xx_hal::{
    delay::Delay,
    gpio,
    gpio::{Floating, Input},
    pac,
    prelude::*,
};
//use dht_hal_drv::{dht_read, dht_split_init, dht_split_read, DhtError, DhtType, DhtValue};
use embedded_hal::digital::v2::{InputPin, OutputPin};

use tmledkey_hal_drv as tm;

#[entry]
fn main() -> ! {
    // Get access to the core peripherals from the cortex-m crate
    let cp = cortex_m::Peripherals::take().unwrap();
    // Get access to the device specific peripherals from the peripheral access crate
    let dp = pac::Peripherals::take().unwrap();

    // Take ownership over the raw flash and rcc devices and convert them into the corresponding
    // HAL structs
    let mut flash = dp.FLASH.constrain();
    // dp.RCC.cfgr.sysclk(1.mhz());
    let mut rcc = dp.RCC.constrain();

    // Freeze the configuration of all the clocks in the system and store
    // the frozen frequencies in `clocks`
    let clocks = rcc.cfgr.freeze(&mut flash.acr);

    // Acquire the GPIOC peripheral
    let mut gpioc = dp.GPIOC.split(&mut rcc.apb2);

    // Configure gpio C pin 13 as a push-pull output. The `crh` register is passed to the function
    // in order to configure the port. For pins 0-7, crl should be passed instead.
    let mut led = gpioc.pc13.into_push_pull_output(&mut gpioc.crh);
    // Configure the syst timer to trigger an update every second
    // let mut timer = Timer::syst(cp.SYST, 1.hz(), clocks);
    let mut delay = Delay::new(cp.SYST, clocks);

    let mut gpiob = dp.GPIOB.split(&mut rcc.apb2);
    let mut tm_clk = gpiob.pb6.into_push_pull_output(&mut gpiob.crl);
    let mut tm_dio = gpiob.pb7.into_open_drain_output(&mut gpiob.crl);

    tm_clk.set_high();
    tm_dio.set_high();

    hprintln!("DIO pin state {}", tm_dio.is_high().unwrap());

    let mut bus_delay = || delay.delay_us(tm::BUS_DELAY_US);

    let r = tm::tm_send_bytes(
        &mut tm_dio,
        &mut tm_clk,
        &mut bus_delay,
        &[tm::COM_DATA_ADDRESS_ADD],
    );
    hprintln!("Display initialized: {:?}", r);

    let r = tm::tm_send_bytes(
        &mut tm_dio,
        &mut tm_clk,
        &mut bus_delay,
        &[tm::COM_DISPLAY_ON],
    );
    hprintln!("Brightness Init {:?}", r);

    let mut nums: [u8; 5] = [tm::COM_ADDRESS | 0, 1, 2, 3, 4];
    let mut iter = 0;
    loop {
        let mut bts: [u8; 5] = [0; 5];
        bts[0] = nums[0];
        for i in 1..nums.len() {
            bts[i] = tm::CHARS[(nums[i] as usize % tm::CHARS.len())];
        }

        let b = iter % 8;
        let r = tm::tm_send_bytes(
            &mut tm_dio,
            &mut tm_clk,
            &mut || delay.delay_us(tm::BUS_DELAY_US),
            &[tm::COM_DISPLAY_ON | iter],
        );

        let pr = tm::tm_send_bytes(
            &mut tm_dio,
            &mut tm_clk,
            &mut || delay.delay_us(tm::BUS_DELAY_US),
            &bts,
        );

        let read = tm::tm_read_byte(&mut tm_dio, &mut tm_clk, &mut || {
            delay.delay_us(tm::BUS_DELAY_US)
        });

        match read {
            Ok(byte) => hprintln!("Byte readed: {:04b}_{:04b}", byte>>4,  byte & 0xF),
            Err(e) => hprintln!("Read error {:?}", e),
        };

        delay.delay_ms(250_u16);
        for i in 1..nums.len() {
            nums[i] = nums[i] + 1;
        }
        if iter % 2 == 0 {
            led.set_low();
        } else {
            led.set_high();
        }
        iter += 1;
    }
}
