# OreSat System Controller

This is the repo for the Rust version of the OreSat system controller.

## Ubuntu/Debian Setup

Install Rust using rustup (the Rust toolchain installer):

```
$ curl https://sh.rustup.rs -sSf | sh
```
complete setup using the onscreen instructions.

Install the following packages:

```
$ sudo apt-get install   gcc-arm-none-eabi   gdb-arm-none-eabi   minicom   openocd
$ sudo cargo install xargo
```

Tell rustup to use rust-nightly toolchain for the syscon directory:

```
.../syscon-rs$ rustup override set nightly-x86_64-unknown-linux-gnu
```

if that does not work, use `rustup toolchain list` to see the available toolchains.

## Build and flash

Build for the stm32f411 using the cargo wrapper xargo so that libcore is built for embedded chips from source.

```
.../syscon-rs$ xargo build
```

Start openocd, and leave it running in the background (or in a seprate terminal), then open gdb:

```
.../syscon-rs$ openocd -f interface/stlink-v2-1.cfg -f target/stm32f4x.cfg
```

```
.../syscon-rs$ arm-none-eabi-gdb -q target/thumbv7em-none-eabihf/debug/oresat-syscon
```

gdb will pause at a temporary breakpoint, type 'c' to continue. 

## What is all this?

A brief intro to those new to Rust and/or embedded development:

- Rustup is the Rust toolchain installer, it will install the latest version of Rust, and manages the various toolchains and targets that are available.
- You also need the arm-eabi versions of gcc and gdb. The eabi is the embedded ABI, essentially referring to the ARM interface for running code on the chip with no intermediate kernel.
- Openocd is the open on chip debugger. This program talks to the programming interface located either on the development board, or on the JTAG adapter connected to a production board. OpenOCD also provides the interface gdb needs to communicate with the chip.
- If you have skimmed over a Rust tutorial, you should be familiar with cargo. xargo is a wrapper for cargo that builds libcore for ARM since cargo does not do this on its own. 