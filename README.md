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
User frienly API would be implemented for popular controller models.


# Project is in early development stage

Right now it is working only with TM1637 using low level function to send bytes.

Tested on hardware (look into examples folder):
 * Raspberry Pi
 * STM32 blue pill


# Licensing
This product is licenses under **almost MIT license** but with plumbus exception.

