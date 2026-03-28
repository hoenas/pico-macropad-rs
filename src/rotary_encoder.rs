pub struct RotaryEncoder {
    pub value: isize,
    pub invert_direction: bool,
    last_pin_a_value: bool,
    last_pin_b_value: bool,
}

impl RotaryEncoder {
    pub fn update(&mut self, pin_a: bool, pin_b: bool) -> isize {
        // https://www.allaboutcircuits.com/projects/how-to-use-a-rotary-encoder-in-a-mcu-based-project/
        if (self.last_pin_a_value && !pin_a && self.last_pin_b_value && pin_b)
            || (!self.last_pin_a_value && !pin_a && !self.last_pin_b_value && pin_b)
            || (!self.last_pin_a_value && pin_a && !self.last_pin_b_value && !pin_b)
            || (self.last_pin_a_value && pin_a && !self.last_pin_b_value && pin_b)
        {
            self.value += 1;
        } else {
            self.value -= 1;
        }
        self.value
    }
}
