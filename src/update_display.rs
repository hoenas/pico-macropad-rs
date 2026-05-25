use alloc::{format, string::String, vec::Vec};
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
        let index = amount % (len + ROTATION_SPACER.len());
        let (start, end) = new_string.split_at(index);
        new_string = format!("{}{}", end, start);
        new_string.chars().take(length).collect()
    }
}

struct DisplayData {
    pub text: String,
    pub icon: Option<Vec<u8>>,
}

trait GetDisplayData {
    fn get_draw_data(&self) -> DisplayData;
}

impl GetDisplayData for crate::ButtonConfig {
    fn get_draw_data(&self) -> DisplayData {
        DisplayData {
            text: self.display_text.clone(),
            icon: self.display_icon.clone(),
        }
    }
}

impl GetDisplayData for crate::MenuEncoderConfig {
    fn get_draw_data(&self) -> DisplayData {
        DisplayData {
            text: self.display_text.clone(),
            icon: self.display_icon.clone(),
        }
    }
}

impl GetDisplayData for crate::EncoderConfig {
    fn get_draw_data(&self) -> DisplayData {
        DisplayData {
            text: self.display_text.clone(),
            icon: self.display_icon.clone(),
        }
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
    let origin = display.bounding_box().top_left;
    // Draw standard encoders
    for (i, encoder) in config.encoders.iter().enumerate() {
        let x = (26 * i) as i32;
        let top_left = origin + Point::new(x, 0);
        draw_icon_cell(display, encoder.get_draw_data(), top_left, rotation_counter);
    }
    // Draw menu encoder
    let x = (26 * 5) as i32;
    let top_left = origin + Point::new(x, 0);
    draw_icon_cell(
        display,
        config.menu_encoder.get_draw_data(),
        top_left,
        rotation_counter,
    );
    // Draw buttons
    for row in 0..2 {
        for col in 0..5 {
            let idx = row * 5 + col;
            let button = &config.buttons[idx];
            let x = (26 * col) as i32;
            let y = 23 + (row * 23) as i32;
            let top_left = origin + Point::new(x, y);
            draw_icon_cell(display, button.get_draw_data(), top_left, rotation_counter);
        }
    }
    display.flush().unwrap();
}

fn draw_icon_cell<DI>(
    display: &mut DI,
    element: DisplayData,
    top_left: Point,
    rotation_counter: usize,
) where
    DI: DrawTarget<Color = BinaryColor>,
    DI::Error: core::fmt::Debug,
{
    // Draw icon if it exists and is validf
    if let Some(icon_bytes) = &element.icon {
        if let Ok(bmp) = tinybmp::Bmp::<BinaryColor>::from_slice(icon_bytes) {
            bmp.draw(&mut display.translated(top_left)).unwrap();
            return;
        }
    }
    // If there is no icon or the icon failed to load, draw rotating text instead
    Text::with_alignment(
        element.text.rotate(rotation_counter, 3).as_str(),
        top_left + Point::new(0, 13),
        CHARACTER_STYLE,
        Alignment::Center,
    )
    .draw(display)
    .unwrap();
}
