use stm32f4::stm32f429::TIM5;

use crate::{
    button::enable_user_button, enable_interrupt, interrupt, ArmPeripherals, ClockSpeeds,
    Peripherals,
};

pub fn initialize_stopwatch(
    stm_peripherals: &Peripherals,
    arm_peripherals: &mut ArmPeripherals,
    frequencies: &ClockSpeeds,
) {
    enable_user_button(&stm_peripherals, arm_peripherals);
    initialize_stopwatch_timer(&stm_peripherals.TIM5, arm_peripherals, frequencies, 100);
}

fn initialize_stopwatch_timer(
    tim5: &TIM5,
    arm_peripherals: &mut ArmPeripherals,
    frequencies: &ClockSpeeds,
    resolution: u32,
) {
    tim5.cr1.write(|w| w.urs().set_bit()); // interrupts only from overflow
    tim5.arr
        .write(|w| w.arr().variant(frequencies.tim1clk / resolution));
    tim5.egr.write(|w| w.ug().update()); // clear state
    tim5.dier.write(|w| w.uie().enabled()); // enable interrupt generation
    tim5.cr1.modify(|_, w| w.cen().disabled()); // disable timer
    enable_interrupt(arm_peripherals, interrupt::TIM5, 40);
}

#[interrupt]
unsafe fn TIM5() {
    Peripherals::steal().TIM5.sr.write(|w| w.uif().clear_bit());
    inc_seconds();
}

static mut CENTI_SECONDS: u16 = 0;
static mut TIM5_COUNTING: bool = false;
static mut TOGGLE_TIMER: bool = false;

#[inline(always)]
pub fn should_toggle_tim5() -> bool {
    unsafe { TOGGLE_TIMER }
}

#[inline(always)]
pub fn set_should_toggle_tim5(val: bool) {
    unsafe {
        TOGGLE_TIMER = val;
    }
}

#[inline(always)]
pub fn is_tim5_counting() -> bool {
    unsafe { TIM5_COUNTING }
}

#[inline(always)]
pub fn set_tim5_counting(val: bool) {
    unsafe {
        TIM5_COUNTING = val;
    }
}

#[inline(always)]
pub fn get_seconds() -> u16 {
    unsafe { CENTI_SECONDS }
}

#[inline(always)]
fn inc_seconds() {
    unsafe {
        CENTI_SECONDS += 1;
    }
}
