# Titan Micro LED controller driver
[![crates.io](https://img.shields.io/crates/v/tmledkey-hal-drv.svg)](https://crates.io/crates/tmledkey-hal-drv)
[![Released API docs](https://docs.rs/tmledkey-hal-drv/badge.svg)](https://docs.rs/tmledkey-hal-drv)

Titan Micro is a Chinese manufacturer that produce several type of controllers for [7 segment LED displays](https://en.wikipedia.org/wiki/Seven-SEG_display) with additional keyboard key scan functionality.

At least next controller variants are exist on the market:
 * TM1636 - 2 wire interface, 4 displays, 16 keys 
 * TM1637 (popular) - 2 wire interface, 6 displays, 16 keys
 * TM1638 (popular) - 3 wire interface, 8 displays (10 segments), 24 keys
 * TM1639 - 3 wire interface, 8 displays (12 segments ?), 8 keys
 * TM1640 - 2 wire interface, 16 displays, no keys

This driver implements low level functions to send/read data with 2 or 3 wire interface.
User friendly API would be implemented for popular controller models later.


# Project status and future plans

Available functionality:
 * Support 2 and 3 wire interfaces, tested on TM1637 and TM1698
 * Writing bytes to MCU
 * Reading key scan bytes from MCU
 * Basic utility and animation features are present
 
Hardware crate was tested on:
 * TM1637 clock module
 * TM1638 module with 8 displays, 8 buttons and 8 additional LEDs
 * STM32 Blue Pill
 * Raspberry Pi

Current functionality looks stable, but implementation is extremely low level.
That is mostly because I see no reason to do more friendly API 
until HAL and it's implementations would stabilize.
Right now my goal is to keep it stable and working between HAL updates.

I really do hope that complexity of API is not a big issue. Current HAL state and embedded programming 
is suited only for hardcore, crazy, masochist developers who should be OK with my code.

# Examples

This is how code from examples works. 

Click on image to view animation.

<a href="https://rust-rum.github.io/tmledkey-hal-drv/tm1637.gif" target="_blank"><img alt="TM1637 example" src="https://rust-rum.github.io/tmledkey-hal-drv/tm1637.jpg" /></a>

<a href="https://rust-rum.github.io/tmledkey-hal-drv/tm1638.gif" target="_blank"><img  alt="TM1638 example" src="https://rust-rum.github.io/tmledkey-hal-drv/tm1638.jpg" /></a>

# Licensing
This product is licenses under **almost MIT license** but with plumbus exception.

