/// display module
/// draws digits on the display when requested

use stm32f4::stm32f429::{GPIOA, GPIOE, TIM2, Peripherals, interrupt};

use crate::stopwatch::get_seconds;
use crate::{enable_interrupt, ArmPeripherals, ClockSpeeds};

/// initialize all peripherals involved in driving the display
pub fn initialize_display(
    stm_peripherals: &Peripherals,
    arm_peripherals: &mut ArmPeripherals,
    frequencies: &ClockSpeeds,
    display_timer_resolution: u32,
) {
    initialize_display_timer(
        &stm_peripherals.TIM2,
        arm_peripherals,
        frequencies,
        display_timer_resolution,
    );
    enable_display(stm_peripherals);
}

/// set up timer which drives display rendering
fn initialize_display_timer(
    tim2: &TIM2,
    arm_peripherals: &mut ArmPeripherals,
    frequencies: &ClockSpeeds,
    resolution: u32,
) {
    tim2.cr1.write(|w| w.urs().set_bit());
    tim2.arr
        .write(|w| w.arr().variant(frequencies.tim1clk / resolution));
    tim2.egr.write(|w| w.ug().update());
    tim2.dier.write(|w| w.uie().enabled());
    tim2.cr1.modify(|_, w| w.cen().disabled());
    enable_interrupt(arm_peripherals, interrupt::TIM2, 50);
    tim2.cr1.modify(|_, w| w.cen().set_bit());
}

/// enable the GPIO pins used to drive display
fn enable_display(stm_peripherals: &Peripherals) {
    // digit selection pins
    stm_peripherals.GPIOA.moder.modify(|_, w| {
        w.moder3()
            .output()
            .moder5()
            .output()
            .moder6()
            .output()
            .moder7()
            .output()
    });

    // digit drawing pins
    stm_peripherals.GPIOE.moder.write(|w| {
        w.moder0()
            .output()
            .moder2()
            .output()
            .moder7()
            .output()
            .moder8()
            .output()
            .moder9()
            .output()
            .moder11()
            .output()
            .moder13()
            .output()
            .moder15()
            .output()
    })
}

/// constants are determined by the register values
/// required to select & draw digits according to
/// the pins used in this project.

const SELECT_FIRST_DIGIT: u32 = 0x20;  // pin A5
const SELECT_SECOND_DIGIT: u32 = 0x40; // pin A6
const SELECT_THIRD_DIGIT: u32 = 0x80;  // pin A7
const SELECT_FOURTH_DIGIT: u32 = 0x8;  // pin A3

const SEGMENT_A: u32 = 0x1;            // pin E0
const SEGMENT_B: u32 = 0x4;            // pin E2
const SEGMENT_C: u32 = 0x80;           // pin E7
const SEGMENT_D: u32 = 0x100;          // pin E8
const SEGMENT_E: u32 = 0x200;          // pin E9
const SEGMENT_F: u32 = 0x800;          // pin E11
const SEGMENT_G: u32 = 0x2000;         // pin E13

const DECIMAL_POINT: u32 = 0x8000;     // pin E15

const DIGIT_ONE: u32 = SEGMENT_B | SEGMENT_C;
const DIGIT_TWO: u32 = SEGMENT_A | SEGMENT_B | SEGMENT_G | SEGMENT_E | SEGMENT_D;
const DIGIT_THREE: u32 = (DIGIT_TWO ^ SEGMENT_E) | SEGMENT_C;
const DIGIT_FOUR: u32 = DIGIT_ONE | SEGMENT_F | SEGMENT_G;
const DIGIT_FIVE: u32 = (DIGIT_THREE ^ SEGMENT_B) | SEGMENT_F;
const DIGIT_SIX: u32 = DIGIT_FIVE | SEGMENT_E;
const DIGIT_SEVEN: u32 = DIGIT_ONE | SEGMENT_A;
const DIGIT_EIGHT: u32 = DIGIT_SIX | SEGMENT_B;
const DIGIT_NINE: u32 = DIGIT_SEVEN | SEGMENT_F | SEGMENT_G;
const DIGIT_ZERO: u32 = DIGIT_EIGHT ^ SEGMENT_G;

/// digit selector values
const DIGITS: [u32; 4] = [
    SELECT_FIRST_DIGIT,
    SELECT_SECOND_DIGIT,
    SELECT_THIRD_DIGIT,
    SELECT_FOURTH_DIGIT,
];

/// digit symbol values
const DIGIT_SYMBOLS: [u32; 10] = [
    DIGIT_ZERO,
    DIGIT_ONE,
    DIGIT_TWO,
    DIGIT_THREE,
    DIGIT_FOUR,
    DIGIT_FIVE,
    DIGIT_SIX,
    DIGIT_SEVEN,
    DIGIT_EIGHT,
    DIGIT_NINE,
];

/// draws a single digit in the place determined by the digit
/// selector pins
fn draw_digit(digit_pins: &GPIOE, val: u16, draw_decimal: bool) {
    let index = usize::from(val);
    let bits = if draw_decimal {
        DIGIT_SYMBOLS[index] | DECIMAL_POINT
    } else {
        DIGIT_SYMBOLS[index]
    };
    unsafe {
        digit_pins.odr.write(|w| w.bits(bits));
    }
}

/// selects the digit to render and which place in which
/// to render it
pub fn render_display(selector_pins: &GPIOA, digit_pins: &GPIOE) {
    static mut CURRENT_DIGIT: u8 = 0;
    static mut TEMP_NUMBER: u16 = 0;
    unsafe {
        if CURRENT_DIGIT == 0 {
            TEMP_NUMBER = get_seconds()
        }
        let digit = TEMP_NUMBER % 10;
        TEMP_NUMBER /= 10;
        selector_pins
            .odr
            .write(|w| w.bits(DIGITS[usize::from(3 - CURRENT_DIGIT)]));
        draw_digit(digit_pins, digit, CURRENT_DIGIT == 2);
        CURRENT_DIGIT = (CURRENT_DIGIT + 1) & 3;
    }
}

/// tim2 interrupt signals `main` to draw the display
#[interrupt]
unsafe fn TIM2() {
    Peripherals::steal().TIM2.sr.write(|w| w.uif().clear_bit());
    set_should_render_display(true);
}

/// Static variables and accessor functions

static mut SHOULD_RENDER_DISPLAY: bool = false;

#[inline(always)]
pub fn should_render_display() -> bool {
    unsafe { SHOULD_RENDER_DISPLAY }
}

#[inline(always)]
pub fn set_should_render_display(val: bool) {
    unsafe {
        SHOULD_RENDER_DISPLAY = val;
    }
}
