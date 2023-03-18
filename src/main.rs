#![no_main]
#![no_std]

use stm32f4xx_hal::{pac::Peripherals, interrupt as interrupt_enum, interrupt};
use cortex_m_rt::entry;
use stm32f439zi_startup::{ClockInit, init_clocks, settings::*};
use cortex_m::{peripheral::NVIC, Peripherals as ArmPeripherals};

#[entry]
fn main() -> ! {
    let stm_peripherals = Peripherals::take().unwrap();
    let mut arm_peripherals = cortex_m::Peripherals::take().unwrap();

    initialize_peripherals(&stm_peripherals, &mut arm_peripherals);

    stm_peripherals.TIM2.cr1.modify(|_, w| w.cen().set_bit());

    loop {
        cortex_m::asm::wfi();
    }
}

fn initialize_peripherals(stm_peripherals: &Peripherals, arm_peripherals: &mut ArmPeripherals) -> () {

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
    let _speeds = init_clocks(SETTINGS, stm_peripherals);

    stm_peripherals.RCC.ahb1enr.modify(|_, w| w
        .gpioaen().enabled()
        .gpioben().enabled()
        .gpiocen().enabled()
        .gpioden().enabled()
        .gpioeen().enabled()
        .dma1en().enabled()
        .dma2en().enabled()
    );
    stm_peripherals.RCC.apb1enr.modify(|_, w| w
        .tim2en().enabled()
        .tim5en().enabled()
    );
    stm_peripherals.RCC.apb2enr.modify(|_, w| w
        .tim10en().enabled()
    );

    stm_peripherals.GPIOB.moder.modify(|_,w| w
        .moder0().output()
        .moder5().output()
        .moder7().output()
        .moder14().output()
    );

    stm_peripherals.TIM2.cr1.modify(|_, w| w.arpe().set_bit().urs().set_bit());
    stm_peripherals.TIM2.arr.write(|w| w.arr().bits(8399999));
    stm_peripherals.TIM2.egr.write(|w| w.ug().set_bit());
    stm_peripherals.TIM2.dier.modify(|_, w| w.uie().set_bit());
    
    unsafe {
        arm_peripherals.NVIC.set_priority(interrupt_enum::TIM2, 1);
        NVIC::unmask(interrupt_enum::TIM2)
    }

}

#[interrupt]
unsafe fn TIM2() {
    Peripherals::steal().TIM2.sr.write(|w| w.uif().clear_bit());
}