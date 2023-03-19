# Bare-metal STM32 Project 1.5: Rusty Stopwatch

In these projects, I implement common or interesting functions using
an STM32F439ZI MCU on a Nucleo-144 board, from the ground up. I do not
use any Hardware Abstraction Layer or pre-written drivers. I use the
`cortex-m` crate which provides linker scripts and startup assembly code,
and the `stm32f4` crate which provides direct register access.

My hand-rolled board initialization procedure can be found in a [companion crate](https://github.com/jonah-saltzman/stm32f439_startup)

## Stopwatch

![stopwatch](https://i.imgur.com/WD1U2QS.jpg)

This project 1.5 is stopwatch 2.0, re-written in Rust. My [previous stopwatch project](https://github.com/jonah-saltzman/bare-metal-stopwatch/)
was written in C. Besides re-writing in Rust, I made the following improvements:
- Button de-bouncing: previously the user button was associated with an interrupt on
the button's rising trigger trigger. If the button bounced, the stopwatch could be
started and stopped instantaneously. This time, I wrote a state machine to de-bounce
the button's input in [/src/button.rs](/src/button.rs) taking inspiration from
[this explanation](https://www.eeweb.com/debouncing-push-buttons-using-a-state-machine-approach/)
- Smaller ISRs: Previously, ISRs performed too much functionality, including rendering
the display and toggling timers. Now, ISRs only set flags which are checked in the `main`
loop and drive functionality there.
- Static variables: Stopwatch in C used global static variables everywhere, and accessed
them directly. Now, statics are moved to inside functions when possible, and otherwise
are only accessed through accessor functions, making these variables easier to reason about.
- Logical organization: instead of grouping funcionality by hardware/software feature (i.e.
`interrupts.c` and `timers.c`), I have grouped Rust-stopwatch into modules according to
high-level function, such as [`button.rs`](/src/button.rs) and [`display.rs`](/src/display.rs).

Stopwatch starts counting time when the user presses the button, with a resolution
of 1/100th of a second, and displays the current time on a 4-digit display. The
stopwatch can be stopped and started again with the button, or reset with the reset
button.

[YouTube link](https://www.youtube.com/shorts/_y3RYFs5QW8)

See the [C-based repository](https://github.com/jonah-saltzman/bare-metal-stopwatch/) for a detailed explanation of all the functionality within stopwatch.

### Usage
Build with `cargo build --target thumbv7em-none-eabihfl`. To program the MCU,
I use openocd and gdb. The [cargo config](/.cargo/config.toml) and [gdb config](/openocd.gdb)
are included in this repository.