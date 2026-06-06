use embedded_hal::digital::InputPin;
use rotary_encoder_hal::{DefaultPhase, Direction, Rotary};
use rp2040_hal::gpio::{Function, Interrupt, PinId, PullType};

pub trait InterruptPin: InputPin {
    fn clear_interrupt(&mut self, interrupt: Interrupt);
}

impl<I, F, P> InterruptPin for rp2040_hal::gpio::Pin<I, F, P>
where
    I: PinId,
    F: Function,
    P: PullType,
    rp2040_hal::gpio::Pin<I, F, P>: InputPin,
{
    fn clear_interrupt(&mut self, interrupt: Interrupt) {
        rp2040_hal::gpio::Pin::clear_interrupt(self, interrupt)
    }
}

pub struct RotaryEncoder<A: InterruptPin, B: InterruptPin> {
    inner: Rotary<A, B, DefaultPhase>,
}

impl<A: InterruptPin, B: InterruptPin> RotaryEncoder<A, B> {
    pub fn new(pin_a: A, pin_b: B) -> Self {
        Self {
            inner: Rotary::new(pin_a, pin_b),
        }
    }

    pub fn update(&mut self) -> Direction {
        self.inner.update().unwrap_or(Direction::None)
    }

    pub fn clear_interrupt(&mut self) {
        let (a, b) = self.inner.pins();
        a.clear_interrupt(Interrupt::EdgeHigh);
        a.clear_interrupt(Interrupt::EdgeLow);
        b.clear_interrupt(Interrupt::EdgeHigh);
        b.clear_interrupt(Interrupt::EdgeLow);
    }
}
