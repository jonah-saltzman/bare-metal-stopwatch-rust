// button de-bouncing module

use cortex_m_rt::exception;

use crate::{stopwatch::set_should_toggle_tim5, ArmPeripherals, Peripherals};

const MIN_BUTTON_COUNT: u8 = 5;

/// button debouncing state machine
/// structs represent different states
/// 
/// Waiting: waiting for a button press
/// Detected: press detected, validate press
/// WaitForRelease: press validated, waiting for button release
pub struct ButtonStateMachine<S> {
    state: S,
}

impl ButtonStateMachine<ButtonWaiting> {
    fn new() -> Self {
        ButtonStateMachine {
            state: ButtonWaiting::new(),
        }
    }
}

pub struct ButtonWaiting {}

impl ButtonWaiting {
    fn new() -> Self {
        ButtonWaiting {}
    }
}

pub struct ButtonDetected {
    tick_count: u8,
}

impl From<ButtonStateMachine<ButtonWaiting>> for ButtonStateMachine<ButtonDetected> {
    fn from(_value: ButtonStateMachine<ButtonWaiting>) -> ButtonStateMachine<ButtonDetected> {
        ButtonStateMachine {
            state: ButtonDetected { tick_count: 0 },
        }
    }
}

pub struct ButtonWaitForRelease {}

impl From<ButtonStateMachine<ButtonDetected>> for ButtonStateMachine<ButtonWaitForRelease> {
    fn from(
        _value: ButtonStateMachine<ButtonDetected>,
    ) -> ButtonStateMachine<ButtonWaitForRelease> {
        set_should_toggle_tim5(true);
        ButtonStateMachine {
            state: ButtonWaitForRelease {},
        }
    }
}

impl From<ButtonStateMachine<ButtonDetected>> for ButtonStateMachine<ButtonWaiting> {
    fn from(_value: ButtonStateMachine<ButtonDetected>) -> ButtonStateMachine<ButtonWaiting> {
        ButtonStateMachine {
            state: ButtonWaiting::new(),
        }
    }
}

impl From<ButtonStateMachine<ButtonWaitForRelease>> for ButtonStateMachine<ButtonWaiting> {
    fn from(_value: ButtonStateMachine<ButtonWaitForRelease>) -> ButtonStateMachine<ButtonWaiting> {
        ButtonStateMachine {
            state: ButtonWaiting::new(),
        }
    }
}

/// ButtonMachineWrapper implements associated `step` function
/// to transition between states
/// 
/// Side effect (toggling tim5) occurs in transition from 
/// Detected state to WaitForRelease state
pub enum ButtonMachineWrapper {
    Waiting(ButtonStateMachine<ButtonWaiting>),
    Detected(ButtonStateMachine<ButtonDetected>),
    Releasing(ButtonStateMachine<ButtonWaitForRelease>),
}

impl ButtonMachineWrapper {
    fn new() -> ButtonMachineWrapper {
        ButtonMachineWrapper::Waiting(ButtonStateMachine::new())
    }

    pub fn step(self, is_high: bool) -> Self {
        match self {
            Self::Waiting(state) => {
                if is_high {
                    Self::Detected(state.into())
                } else {
                    Self::Waiting(state)
                }
            }
            Self::Detected(mut state) => {
                if is_high {
                    state.state.tick_count += 1;
                    if state.state.tick_count >= MIN_BUTTON_COUNT {
                        Self::Releasing(state.into())
                    } else {
                        Self::Detected(state)
                    }
                } else {
                    Self::Waiting(state.into())
                }
            }
            Self::Releasing(state) => {
                if is_high {
                    Self::Releasing(state)
                } else {
                    Self::new()
                }
            }
        }
    }
}

/// enables all peripherals required to drive
/// the user button
pub fn enable_user_button(
    stm_peripherals: &Peripherals,
    arm_peripherals: &mut ArmPeripherals,
) -> ButtonMachineWrapper {
    stm_peripherals
        .GPIOC
        .moder
        .modify(|_, w| w.moder13().input());
    arm_peripherals.SYST.set_reload(200_000);
    arm_peripherals.SYST.clear_current();
    arm_peripherals.SYST.enable_interrupt();
    arm_peripherals.SYST.enable_counter();
    ButtonMachineWrapper::new()
}

/// SysTick exception signals `main` to step the
/// button-debounce state machine
#[exception]
unsafe fn SysTick() {
    set_should_step_machine(true);
}

/// Static variables and accessor functions

static mut SHOULD_STEP_MACHINE: bool = false;

pub fn set_should_step_machine(val: bool) {
    unsafe {
        SHOULD_STEP_MACHINE = val;
    }
}

pub fn should_step_machine() -> bool {
    unsafe { SHOULD_STEP_MACHINE }
}
