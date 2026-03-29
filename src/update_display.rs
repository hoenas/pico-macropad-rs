use alloc::{fmt::format, format, string::String};
use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyle},
    pixelcolor::BinaryColor,
    prelude::*,
    text::{Alignment, Text},
};
use rp2040_hal::{
    gpio::{
        bank0::{Gpio26, Gpio27},
        FunctionI2c, Pin, PullUp,
    },
    I2C,
};
use rp_pico::pac::I2C1;
use sh1106::{mode::GraphicsMode, prelude::I2cInterface};

use crate::MacroConfig;

pub const CHARACTER_STYLE: MonoTextStyle<BinaryColor> =
    MonoTextStyle::new(&FONT_6X10, BinaryColor::On);

const ROTATION_SPACING: usize = 3;
const ROTATION_SPACER: &str = &"   ";

trait Rotate {
    fn rotate(&self, amount: usize, length: usize) -> String;
}

impl Rotate for String {
    fn rotate(&self, amount: usize, length: usize) -> String {
        let len = self.len();
        if len <= length {
            return self.clone();
        }
        let mut new_string = format!("{}{}", self, ROTATION_SPACER);
        let index = amount % (len + ROTATION_SPACING);
        let (start, end) = new_string.split_at(index);
        new_string = format!("{}{}", end, start);
        return new_string.chars().take(length).collect();
    }
}

pub fn update_display(
    display: &mut GraphicsMode<
        I2cInterface<
            I2C<
                I2C1,
                (
                    Pin<Gpio26, FunctionI2c, PullUp>,
                    Pin<Gpio27, FunctionI2c, PullUp>,
                ),
            >,
        >,
    >,
    config: &MacroConfig,
    rotation_counter: usize,
) {
    Text::with_alignment(
        config.name.rotate(rotation_counter, 21).as_str(),
        display.bounding_box().top_left + Point::new(0, 10),
        CHARACTER_STYLE,
        Alignment::Left,
    )
    .draw(display)
    .unwrap();
    Text::with_alignment(
        format!(
            "{:^3}|{:^3}        {:^4}",
            config.encoder1.display_text.rotate(rotation_counter, 4),
            config.encoder2.display_text.rotate(rotation_counter, 4),
            config.menu_encoder.display_text.rotate(rotation_counter, 4)
        )
        .as_str(),
        display.bounding_box().top_left + Point::new(0, 30),
        CHARACTER_STYLE,
        Alignment::Left,
    )
    .draw(display)
    .unwrap();
    Text::with_alignment(
        format!(
            "{:^3}|{:^3}|{:^3}|{:^3}|{:^3}",
            config.button0.display_text.rotate(rotation_counter, 3),
            config.button1.display_text.rotate(rotation_counter, 3),
            config.button2.display_text.rotate(rotation_counter, 3),
            config.button3.display_text.rotate(rotation_counter, 3),
            config.button4.display_text.rotate(rotation_counter, 3)
        )
        .as_str(),
        display.bounding_box().top_left + Point::new(0, 50),
        CHARACTER_STYLE,
        Alignment::Left,
    )
    .draw(display)
    .unwrap();
    Text::with_alignment(
        format!(
            "{:^3}|{:^3}|{:^3}|{:^3}|{:^3}",
            config.button5.display_text.rotate(rotation_counter, 3),
            config.button6.display_text.rotate(rotation_counter, 3),
            config.button7.display_text.rotate(rotation_counter, 3),
            config.button8.display_text.rotate(rotation_counter, 3),
            config.button9.display_text.rotate(rotation_counter, 3)
        )
        .as_str(),
        display.bounding_box().top_left + Point::new(0, 60),
        CHARACTER_STYLE,
        Alignment::Left,
    )
    .draw(display)
    .unwrap();
    display.flush().unwrap();
}
