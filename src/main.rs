#![no_std]
#![no_main]
#![feature(abi_avr_interrupt, sync_unsafe_cell)]

mod system;

use core::cell::RefCell;

use arduino_hal::{
    default_serial, delay_ms,
    hal::usart::Usart0,
    port::{
        mode::{Input, Output, PullUp},
        Pin,
    },
    DefaultClock,
};
use avr_device::interrupt::{self, Mutex};
use panic_halt as _;
use system::{State, System};

type Console = Usart0<DefaultClock>;

static SYSTEM: Mutex<RefCell<Option<System>>> = Mutex::new(RefCell::new(None));
static CONSOLE: Mutex<RefCell<Option<Console>>> = Mutex::new(RefCell::new(None));

fn init_system(led_1: Pin<Output>, led_2: Pin<Output>) {
    interrupt::free(|cs| *SYSTEM.borrow(cs).borrow_mut() = Some(System::new(led_1, led_2)));
}

fn init_console(console: Console) {
    interrupt::free(|cs| {
        *CONSOLE.borrow(cs).borrow_mut() = Some(console);
    });
}

#[arduino_hal::entry]
fn main() -> ! {
    let peripherals = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(peripherals);

    init_console(default_serial!(peripherals, pins, 115200));

    let external_interrupts = peripherals.EXINT;
    let external_interrupt_control = &external_interrupts.eicra;
    let external_interrupt_mask = &external_interrupts.eimsk;

    let buttons = [
        pins.d2.into_pull_up_input().downgrade(),
        pins.d3.into_pull_up_input().downgrade(),
    ];

    init_system(
        pins.d7.into_output().downgrade(),
        pins.d8.into_output().downgrade(),
    );

    // Configure INT0 for falling edge
    external_interrupt_control.write(|w| w.isc0().val_0x02());
    // Configure INT1 for falling edge
    external_interrupt_control.write(|w| w.isc1().val_0x02());

    external_interrupt_mask.write(|w| w.int0().set_bit().int1().set_bit());

    unsafe { interrupt::enable() };

    fn all_unpressed(buttons: &[Pin<Input<PullUp>>]) -> bool {
        buttons.iter().all(|b| b.is_high())
    }

    loop {
        if all_unpressed(&buttons) {
            // Debounce
            delay_ms(50);
            if all_unpressed(&buttons) {
                with_system(|system| {
                    println!("Set 0");
                    system.set_state(State::State0);
                });
            }
        }

        with_system(System::update);
    }
}

fn with_system<F: FnMut(&mut System)>(mut f: F) {
    interrupt::free(|cs| {
        if let Some(system) = SYSTEM.borrow(cs).borrow_mut().as_mut() {
            f(system);
        }
    });
}

// Button 1 interrupt
#[avr_device::interrupt(atmega328p)]
fn INT0() {
    with_system(|system| {
        println!("Set 1");
        system.set_state(State::State1);
    });
}

// Button 2 interrupt
#[avr_device::interrupt(atmega328p)]
fn INT1() {
    with_system(|system| {
        println!("Set 2");
        system.set_state(State::State2);
    });
}

#[macro_export]
macro_rules! println {
    ($($t:tt)*) => {
        interrupt::free(
            |cs| {
                if let Some(console) = CONSOLE.borrow(cs).borrow_mut().as_mut() {
                    let _ = ufmt::uwriteln!(console, $($t)*);
                }
            },
        )
    };
}
