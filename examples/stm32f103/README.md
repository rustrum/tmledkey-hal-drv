# Demo for STM32 Blue Pill board

## Pins

TM1637 2 wire module should be connected as:
 - DIO to PB6
 - CLK to PB5

TM1638 3 wire module use other pins:
 - DIO to PB9
 - CLK to PB8
 - STB to PB6

You can connect 2 modules at the same time but demo would run only for one of them.

## Running with STLink v2

First you should properly start openocd.
All program output would be available here.

```bash
openocd -f interface/stlink-v2.cfg -f target/stm32f1x.cfg
```

Next setup your rustup to use nightly toolchain
```bash
rustup default nightly
```

Now you can run demo for TM1637 clock module by executing
```bash
cargo run --release
```
You should be ended up in (gdb) console
To start demo enter 
```bash
(dbg) continue
```
And do not forget to press Enter

TM1638 demo could be run like this
```bash
cargo run --release --features dioclkstb
```