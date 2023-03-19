#![no_main]
#![no_std]

use cortex_m::{peripheral::NVIC, Peripherals as ArmPeripherals};
use cortex_m_rt::entry;
use stm32f439zi_startup::{init_clocks, settings::*, ClockInit, ClockSpeeds};
use stm32f4xx_hal::{interrupt, pac::Peripherals};

mod button;
mod display;
mod stopwatch;
use display::{
    enable_display, get_should_render_display, initialize_display_timer, render_display,
    set_should_render_display,
};
use stopwatch::{
    initialize_stopwatch, is_tim5_counting, set_should_toggle_tim5, set_tim5_counting,
    should_toggle_tim5,
};

#[entry]
fn main() -> ! {
    let stm_peripherals = Peripherals::take().unwrap();
    let mut arm_peripherals = cortex_m::Peripherals::take().unwrap();

    let frequencies = initialize_peripherals(&stm_peripherals);

    initialize_display_timer(
        &stm_peripherals.TIM2,
        &mut arm_peripherals,
        &frequencies,
        1200,
    );
    initialize_stopwatch(&stm_peripherals, &mut arm_peripherals, &frequencies);
    enable_display(&stm_peripherals);

    // enable the display render clock
    stm_peripherals.TIM2.cr1.modify(|_, w| w.cen().set_bit());

    loop {
        if get_should_render_display() {
            render_display(&stm_peripherals.GPIOA, &stm_peripherals.GPIOE);
            set_should_render_display(false);
        }
        if should_toggle_tim5() {
            if is_tim5_counting() {
                // stop timer
                stm_peripherals.TIM5.cr1.modify(|_, w| w.cen().disabled());
                set_tim5_counting(false);
            } else {
                // reset & start timer
                stm_peripherals.TIM5.egr.write(|w| w.ug().update());
                stm_peripherals.TIM5.cr1.modify(|_, w| w.cen().enabled());
                set_tim5_counting(true);
            }
            set_should_toggle_tim5(false);
        }
        cortex_m::asm::wfi();
    }
}

fn initialize_peripherals(stm_peripherals: &Peripherals) -> ClockSpeeds {
    const SETTINGS: ClockInit = ClockInit {
        pll_source_hse: Some(true),
        sys_source: ClockSource::Pll,
        systick_source: SysTickSource::HclkDiv8,
        timpre: false,
        pll_q: 4,
        pll_p: PLLP::Two,
        pll_n: 160,
        pll_m: 4,
        ahb_pre: AHBFactor::One,
        apb2_pre: APBxFactor::Two,
        apb1_pre: APBxFactor::Four,
    };
    let speeds = init_clocks(SETTINGS, stm_peripherals);

    stm_peripherals.RCC.ahb1enr.modify(|_, w| {
        w.gpioaen()
            .enabled()
            .gpiocen()
            .enabled()
            .gpioeen()
            .enabled()
    });
    stm_peripherals
        .RCC
        .apb1enr
        .modify(|_, w| w.tim2en().enabled().tim5en().enabled());
    // stm_peripherals.RCC.apb2enr.modify(|_, w| w
    //     .tim10en().enabled()
    // );

    speeds
}

/// SAFETY: enabling an interrupt during a critical section could break it
pub fn enable_interrupt(arm_peripherals: &mut ArmPeripherals, intr: interrupt, prio: u8) {
    unsafe {
        arm_peripherals.NVIC.set_priority(intr, prio);
        NVIC::unmask(intr);
    }
}
