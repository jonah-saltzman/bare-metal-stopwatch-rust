#![no_main]
#![no_std]

use cortex_m::{peripheral::NVIC, Peripherals as ArmPeripherals};
use cortex_m_rt::entry;
use stm32f439zi_startup::{init_clocks, settings::*, ClockInit, ClockSpeeds};
use stm32f4::stm32f429::{Peripherals, interrupt};

mod button;
mod display;
mod stopwatch;
use button::{set_should_step_machine, should_step_machine, ButtonMachineWrapper};
use display::{
    initialize_display, render_display, set_should_render_display, should_render_display,
};
use stopwatch::{
    initialize_stopwatch, is_tim5_counting, set_should_toggle_tim5, should_toggle_tim5, start_tim5,
    stop_tim5,
};

#[entry]
fn main() -> ! {
    let stm_peripherals = Peripherals::take().unwrap();
    let mut arm_peripherals = cortex_m::Peripherals::take().unwrap();

    let frequencies = initialize_peripherals(&stm_peripherals);
    initialize_display(&stm_peripherals, &mut arm_peripherals, &frequencies, 1200);
    let mut button_machine =
        initialize_stopwatch(&stm_peripherals, &mut arm_peripherals, &frequencies);

    loop {
        // draw the 4-digit display
        if should_render_display() {
            render_display(&stm_peripherals.GPIOA, &stm_peripherals.GPIOE);
            set_should_render_display(false);
        }
        // tim5 counts time for the stopwatch in 0.01s increments
        if should_toggle_tim5() {
            if is_tim5_counting() {
                stop_tim5(&stm_peripherals.TIM5);
            } else {
                start_tim5(&stm_peripherals.TIM5);
            }
            set_should_toggle_tim5(false);
        }
        // step the button-debouncing state machine
        if should_step_machine() {
            button_machine = ButtonMachineWrapper::step(
                button_machine,
                stm_peripherals.GPIOC.idr.read().idr13().is_high(),
            );
            set_should_step_machine(false);
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
    speeds
}

pub fn enable_interrupt(arm_peripherals: &mut ArmPeripherals, intr: interrupt, prio: u8) {
    unsafe {
        arm_peripherals.NVIC.set_priority(intr, prio);
        NVIC::unmask(intr);
    }
}
