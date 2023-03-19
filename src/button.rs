// todo: button de-bouncing

use crate::{
    enable_interrupt, interrupt, stopwatch::set_should_toggle_tim5, ArmPeripherals, Peripherals,
};

pub fn enable_user_button(stm_peripherals: &Peripherals, arm_peripherals: &mut ArmPeripherals) {
    stm_peripherals
        .SYSCFG
        .exticr4
        .write(|w| w.exti13().variant(0b0010)); // EXTI13 pin C13
    stm_peripherals.EXTI.rtsr.write(|w| w.tr13().enabled()); // enable rising trigger
    //stm_peripherals.EXTI.ftsr.write(|w| w.tr13().enabled()); // enable falling trigger
    stm_peripherals.EXTI.imr.write(|w| w.mr13().unmasked()); // enable interrupt
    enable_interrupt(arm_peripherals, interrupt::EXTI15_10, 30);
}

#[interrupt]
unsafe fn EXTI15_10() {
    Peripherals::steal()
        .EXTI
        .pr
        .modify(|_, w| w.pr13().set_bit());
    set_should_toggle_tim5(true);
}
