use alloc::{format, string::String};
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
const ROTATION_SPACER: &str = "   ";

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
        new_string.chars().take(length).collect()
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
    display.clear();
    Text::with_alignment(
        config.name.rotate(rotation_counter, 21).as_str(),
        display.bounding_box().top_left + Point::new(0, 10),
        CHARACTER_STYLE,
        Alignment::Left,
    )
    .draw(display)
    .unwrap();
    let buttons = [
        &config.button0,
        &config.button1,
        &config.button2,
        &config.button3,
        &config.button4,
        &config.button5,
        &config.button6,
        &config.button7,
        &config.button8,
        &config.button9,
    ];

    let origin = display.bounding_box().top_left;
    Text::with_alignment(
        config.name.as_str(),
        origin + Point::new(0, 10),
        CHARACTER_STYLE,
        Alignment::Left,
    )
    .draw(display)
    .unwrap();

    for row in 0..2 {
        for col in 0..5 {
            let idx = row * 5 + col;
            let button = buttons[idx];
            let x = (22 * col) as i32;
            let y = 24 + (row * 20) as i32;
            let top_left = origin + Point::new(x, y);
            draw_button_cell(display, button, top_left);
        }
    }
    display.flush().unwrap();
}

fn draw_button_cell<DI>(display: &mut DI, button: &crate::ButtonConfig, top_left: Point)
where
    DI: DrawTarget<Color = BinaryColor>,
    DI::Error: core::fmt::Debug,
{
    if let Some(icon_bytes) = &button.display_icon {
        if let Ok(bmp) = tinybmp::Bmp::<BinaryColor>::from_slice(icon_bytes) {
            bmp.draw(&mut display.translated(top_left)).unwrap();
            return;
        }
    }

    Text::with_alignment(
        button.display_text.as_str(),
        top_left + Point::new(10, 14),
        CHARACTER_STYLE,
        Alignment::Center,
    )
    .draw(display)
    .unwrap();
}
