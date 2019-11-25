#![no_std]
#![no_main]
#![feature(lang_items)]

use panic_halt as _;

use cortex_m_rt::{
    entry,
    heap_start
};
use cortex_m_semihosting::hprintln;
use stm32f1xx_hal::{
    delay::Delay,
    pac,
    prelude::*,
};

use embedded_hal::digital::v2::{InputPin, OutputPin};
use core::alloc::Layout;
use alloc_cortex_m::CortexMHeap;

use tmledkey_hal_drv::{
    self as tm,
    demo
};


// Plug in the allocator crate
extern crate alloc;
extern crate alloc_cortex_m;

#[global_allocator]
static ALLOCATOR: CortexMHeap = CortexMHeap::empty();

#[entry]
fn main() -> ! {
    // Initialize the allocator BEFORE you use it
    let start = heap_start() as usize;
    let size = 1024; // in bytes
    unsafe { ALLOCATOR.init(start, size) }


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

    // {
    demo_2wire(&mut delay, &mut led, &mut tm_dio, &mut tm_clk)
    // }

}

fn demo_2wire<LED, DIO,CLK>( 
        delay: &mut Delay,
        led: &mut LED,
        dio: &mut DIO,
        clk: &mut CLK,
) -> ! where
    LED: OutputPin,
    DIO: InputPin + OutputPin,
    CLK: OutputPin 
{
    let delay_time = tm::TM1637_BUS_DELAY_US;

    hprintln!("Starting 2 wire demo (TM1637)");

    let mut demo = demo::Demo::new(4);
    let mut iter = 0;

    let init_res = demo.init_2wire(dio, clk, &mut |d:u16| delay.delay_us(d), delay_time);

    hprintln!("Display initialized {:?}", init_res);

    let mut last_byte = 0_u8;
    loop {
        let read = demo.next_2wire(dio, clk, &mut |d:u16| delay.delay_us(d), delay_time);
        match read {
            Ok(byte) => {
                if byte != last_byte {
                hprintln!("Key scan read: {:04b}_{:04b}", byte>>4,  byte & 0xF);
                last_byte = byte;
                }
            },
            Err(e) => {hprintln!("Key scan read error {:?}", e);},
        };

        delay.delay_ms(100_u32);

        if iter % 2 == 0 {
            led.set_low();
        } else {
            led.set_high();
        }
        iter += 1;
    }
}

// required: define how Out Of Memory (OOM) conditions should be handled
// *if* no other crate has already defined `oom`
#[lang = "oom"]
#[no_mangle]
pub fn rust_oom(layout: Layout) -> ! {
    hprintln!("OOM happens {}", layout.size());
    loop {

    }
}
