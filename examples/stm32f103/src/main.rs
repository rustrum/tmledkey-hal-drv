#![deny(unsafe_code)]
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

use dht_hal_drv::{dht_read, dht_split_init, dht_split_read, DhtError, DhtType, DhtValue};
use embedded_hal::digital::v2::{InputPin, OutputPin};

use tmledkey_hal_drv::*;

// Define types for DHT interface
type DhtHwPin = gpio::gpiob::PB9<Input<Floating>>;
type DhtHwPinCr = gpio::gpiob::CRH;

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

    // DHT pin config
    let mut gpiob = dp.GPIOB.split(&mut rcc.apb2);
    let mut dht_open_drain = gpiob.pb9.into_open_drain_output(&mut gpiob.crh);

    let mut tm_clk = gpiob.pb6.into_push_pull_output(&mut gpiob.crl);
    let mut tm_dio = gpiob.pb7.into_open_drain_output(&mut gpiob.crl);
    // let mut display = TM1637::new(&mut tm_clk, &mut tm_dio, &mut delay);

    // let mut bus_delay = || delay.delay_us(5_u16);

    // bus_delay();

    // hprintln!("DIO pin state {}", tm_dio.is_high().unwrap());
    // bus_delay();
    // tm_bus_stop(&mut tm_dio, &mut tm_clk, &mut bus_delay);
    // bus_delay();
    // for _ in 0..10 {
    //     bus_delay();
    // }

    hprintln!("DIO pin state {}", tm_dio.is_high().unwrap());

    for _ in 0..10 {
        let cmd1 = tm_send_bytes(
            &mut tm_dio,
            &mut tm_clk,
            &mut || delay.delay_us(5_u16),
            &[TM_COM_DISPLAY | 0b100],
        );

        delay.delay_ms(100_u32);

        let cmd2 = tm_send_bytes(
            &mut tm_dio,
            &mut tm_clk,
            &mut || delay.delay_us(5_u16),
            &[TM_COM_ADR | 0, TM_COM_DATA | TM_SEGMENT_1 | TM_SEGMENT_4],
        );

        delay.delay_ms(100_u32);

        hprintln!("CMD1 {:?}", cmd1);
        hprintln!("CMD2 {:?}", cmd2);

        hprintln!("DIO pin state {}", tm_dio.is_high().unwrap());
    }

    loop {
        {
            let readings = dht_read(DhtType::DHT11, &mut dht_open_drain, &mut |d| {
                delay.delay_us(d)
            });
            match readings {
                Ok(res) => {
                    // Long blinks if everything is OK
                    led_blink(&mut led, &mut delay, 250);
                    hprintln!("DHT readins {}C {}%", res.temperature(), res.humidity());
                }
                Err(err) => {
                    // Short blinks on errors
                    for _ in 0..10 {
                        led_blink(&mut led, &mut delay, 25);
                    }
                    hprintln!("DHT ERROR {:?}", err);
                }
            };
            delay.delay_ms(1_000_u32);
        }
    }
}

fn led_blink<Error>(pin: &mut dyn OutputPin<Error = Error>, delay: &mut Delay, ms: u32) {
    pin.set_high();
    delay.delay_ms(ms);
    pin.set_low();
    delay.delay_ms(ms);
}
