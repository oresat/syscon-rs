# OreSat System Controller

## Ubuntu/Debian Setup

Install Rust using rustup (the Rust toolchain installer):

```
$ curl https://sh.rustup.rs -sSf | sh
```
complete setup using the onscreen instructions.

Install the following packages:

```
$ sudo apt-get install   gcc-arm-none-eabi   gdb-arm-none-eabi   minicom   openocd
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
.../syscon-rs$ arm-none-eabi-gdb -q target/thumbv7em-none-eabihf/debug/hello-world
```

gdb will pause at a temporary breakpoint, type 'c' to continue. 