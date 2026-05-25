use alloc::vec;

use crate::*;
pub fn get_example_config() -> MacroConfig {
    MacroConfig {
        name: "example".into(),
        buttons: [
            ButtonConfig {
                display_text: String::from("A Macro Pad"),
                display_icon: Some(include_bytes!("assets/test_icon.bmp").to_vec()),
                keystroke: vec![
                    vec![KeyboardCode::A],
                    vec![KeyboardCode::A],
                    vec![KeyboardCode::A],
                    vec![KeyboardCode::A],
                ],
                ..Default::default()
            },
            ButtonConfig {
                display_text: String::from("B Macro Pad"),
                display_icon: Some(include_bytes!("assets/test_icon.bmp").to_vec()),
                keystroke: vec![
                    vec![KeyboardCode::B],
                    vec![KeyboardCode::B],
                    vec![KeyboardCode::B],
                ],
                ..Default::default()
            },
            ButtonConfig {
                display_text: String::from("C Macro Pad"),
                display_icon: Some(include_bytes!("assets/test_icon.bmp").to_vec()),
                keystroke: vec![vec![KeyboardCode::C], vec![KeyboardCode::C]],
                ..Default::default()
            },
            ButtonConfig {
                display_text: String::from("D Macro Pad"),
                display_icon: Some(include_bytes!("assets/test_icon.bmp").to_vec()),
                keystroke: vec![vec![KeyboardCode::D]],
                ..Default::default()
            },
            ButtonConfig {
                display_text: String::from("E Macro Paaaad"),
                display_icon: Some(include_bytes!("assets/test_icon.bmp").to_vec()),
                keystroke: vec![vec![KeyboardCode::E]],
                ..Default::default()
            },
            ButtonConfig {
                display_text: String::from("F Macro Pad"),
                display_icon: Some(include_bytes!("assets/test_icon.bmp").to_vec()),
                keystroke: vec![vec![KeyboardCode::F]],
                ..Default::default()
            },
            ButtonConfig {
                display_text: String::from("G Macro Pad"),
                display_icon: Some(include_bytes!("assets/test_icon.bmp").to_vec()),
                keystroke: vec![vec![KeyboardCode::G]],
                ..Default::default()
            },
            ButtonConfig {
                display_text: String::from("H Macro Paaaaaaad"),
                display_icon: Some(include_bytes!("assets/test_icon.bmp").to_vec()),
                keystroke: vec![vec![KeyboardCode::H]],
                ..Default::default()
            },
            ButtonConfig {
                display_text: String::from("I Macro Pad"),
                display_icon: Some(include_bytes!("assets/test_icon.bmp").to_vec()),
                keystroke: vec![vec![KeyboardCode::I]],
                ..Default::default()
            },
            ButtonConfig {
                display_text: String::from("J Maaaacro Pad"),
                display_icon: Some(include_bytes!("assets/test_icon.bmp").to_vec()),
                keystroke: vec![
                    vec![KeyboardCode::J],
                    vec![KeyboardCode::K],
                    vec![KeyboardCode::L],
                    vec![KeyboardCode::M],
                    vec![KeyboardCode::N],
                    vec![KeyboardCode::O],
                    vec![KeyboardCode::P],
                    vec![KeyboardCode::Q],
                    vec![KeyboardCode::R],
                    vec![KeyboardCode::S],
                ],
                ..Default::default()
            },
        ],
        menu_encoder: MenuEncoderConfig {
            display_text: String::from("Volume"),
            display_icon: Some(include_bytes!("assets/test_icon.bmp").to_vec()),
            keystroke_left: vec![vec![KeyboardCode::VolumeDown]],
            keystroke_right: vec![vec![KeyboardCode::VolumeUp]],
            ..Default::default()
        },
        encoders: [
            EncoderConfig {
                display_text: String::from("Copy/Paste"),
                keystroke_left: vec![vec![KeyboardCode::LeftControl, KeyboardCode::C]],
                keystroke_right: vec![vec![KeyboardCode::LeftControl, KeyboardCode::V]],
                keystroke_push: vec![vec![KeyboardCode::LeftControl, KeyboardCode::V]],
                display_icon: Some(include_bytes!("assets/test_icon.bmp").to_vec()),
                ..Default::default()
            },
            EncoderConfig {
                display_text: String::from("Copy/Paste"),
                keystroke_left: vec![vec![KeyboardCode::LeftControl, KeyboardCode::C]],
                keystroke_right: vec![vec![KeyboardCode::LeftControl, KeyboardCode::V]],
                keystroke_push: vec![vec![KeyboardCode::LeftControl, KeyboardCode::V]],
                display_icon: Some(include_bytes!("assets/test_icon.bmp").to_vec()),
                ..Default::default()
            },
        ],
        leds: [
            LedConfig { r: 255, g: 0, b: 0 },
            LedConfig { r: 0, g: 255, b: 0 },
            LedConfig { r: 0, g: 0, b: 255 },
            LedConfig {
                r: 255,
                g: 255,
                b: 0,
            },
            LedConfig {
                r: 255,
                g: 0,
                b: 255,
            },
            LedConfig {
                r: 0,
                g: 255,
                b: 255,
            },
            LedConfig {
                r: 255,
                g: 255,
                b: 255,
            },
            LedConfig {
                r: 128,
                g: 128,
                b: 128,
            },
        ],
    }
}
