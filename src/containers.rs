#[derive(Default)]
pub struct Encoder {
    pub value: usize,
    pub value_changed: bool,
    pub delta: isize,
    pub button: bool,
}

impl Encoder {
    pub fn read_delta(&mut self) -> isize {
        let delta = self.delta;
        self.delta = 0;
        delta
    }
}


#[derive(Default)]
pub struct Encoders {
    pub menu_encoder: Encoder,
    pub encoder1: Encoder,
    pub encoder2: Encoder,
}

#[derive(Default)]
pub struct MacroPadButton {
    pub pressed: bool,
    pub pressed_changed: bool,
}

impl MacroPadButton {
    pub fn update(&mut self, new_pressed: bool) {
        if self.pressed != new_pressed {
            self.pressed_changed = true;
            self.pressed = new_pressed;
        } else {
            self.pressed_changed = false;
        }
    }
}


#[derive(Default)]
pub struct MacroPadButtons {
    pub pad0: MacroPadButton,
    pub pad1: MacroPadButton,
    pub pad2: MacroPadButton,
    pub pad3: MacroPadButton,
    pub pad4: MacroPadButton,
    pub pad5: MacroPadButton,
    pub pad6: MacroPadButton,
    pub pad7: MacroPadButton,
    pub pad8: MacroPadButton,
    pub pad9: MacroPadButton,
}

impl MacroPadButtons {
    pub fn any_button_changed(&self) -> bool {
        self.pad0.pressed_changed
            || self.pad1.pressed_changed
            || self.pad2.pressed_changed
            || self.pad3.pressed_changed
            || self.pad4.pressed_changed
            || self.pad5.pressed_changed
            || self.pad6.pressed_changed
            || self.pad7.pressed_changed
            || self.pad8.pressed_changed
            || self.pad9.pressed_changed
    }
}
