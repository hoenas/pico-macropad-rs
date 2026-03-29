use alloc::format;
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
) {
    Text::with_alignment(
        format!(
            "[{}][{}]",
            config.encoder1.display_text, config.encoder2.display_text
        )
        .as_str(),
        display.bounding_box().top_left + Point::new(0, 10),
        CHARACTER_STYLE,
        Alignment::Left,
    )
    .draw(display)
    .unwrap();
    Text::with_alignment(
        format!("[{}]", config.menu_encoder.display_text).as_str(),
        display.bounding_box().top_left + Point::new(90, 10),
        CHARACTER_STYLE,
        Alignment::Left,
    )
    .draw(display)
    .unwrap();
    Text::with_alignment(
        format!(
            "[{}][{}][{}][{}][{}]",
            config.button0.display_text,
            config.button1.display_text,
            config.button2.display_text,
            config.button3.display_text,
            config.button4.display_text
        )
        .as_str(),
        display.bounding_box().top_left + Point::new(0, 40),
        CHARACTER_STYLE,
        Alignment::Left,
    )
    .draw(display)
    .unwrap();
    Text::with_alignment(
        format!(
            "[{}][{}][{}][{}][{}]",
            config.button5.display_text,
            config.button6.display_text,
            config.button7.display_text,
            config.button8.display_text,
            config.button9.display_text
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
