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
    pub pads: [MacroPadButton; 10],
}

impl MacroPadButtons {
    pub fn any_button_changed(&self) -> bool {
        self.pads.iter().any(|x| x.pressed_changed)
    }
}
