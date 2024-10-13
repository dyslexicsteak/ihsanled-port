use arduino_hal::{
    delay_ms,
    port::{mode::Output, Pin},
};

pub(crate) struct System {
    state: State,
    led_1: Pin<Output>,
    led_2: Pin<Output>,
}

impl System {
    pub(crate) fn new(led_1: Pin<Output>, led_2: Pin<Output>) -> Self {
        Self {
            state: State::State0,
            led_1,
            led_2,
        }
    }

    pub(crate) fn set_state(&mut self, new_state: State) {
        self.state = new_state;
        self.update();
    }

    pub(crate) fn update(&mut self) {
        match self.state {
            State::State0 => {
                self.led_1.set_low();
                self.led_2.set_low();
            }
            State::State1 => {
                self.led_1.set_high();
                delay_ms(300);
                self.led_1.set_low();
                self.led_2.set_high();
                delay_ms(300);
                self.led_1.set_high();
            }
            State::State2 => {
                self.led_1.set_high();
                delay_ms(300);
                self.led_2.set_low();
                delay_ms(300);
            }
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) enum State {
    State0,
    State1,
    State2,
}
