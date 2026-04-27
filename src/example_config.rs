use alloc::vec;

use crate::*;
pub fn get_example_config() -> MacroConfig {
    MacroConfig {
        name: "example".into(),
        button0: ButtonConfig {
            display_text: String::from("A"),
            keystroke: vec![
                vec![KeyboardCode::A],
                vec![KeyboardCode::A],
                vec![KeyboardCode::A],
                vec![KeyboardCode::A],
            ],
            ..Default::default()
        },
        button1: ButtonConfig {
            display_text: String::from("B"),
            keystroke: vec![
                vec![KeyboardCode::B],
                vec![KeyboardCode::B],
                vec![KeyboardCode::B],
            ],
            ..Default::default()
        },
        button2: ButtonConfig {
            display_text: String::from("C"),
            keystroke: vec![vec![KeyboardCode::C], vec![KeyboardCode::C]],
            ..Default::default()
        },
        button3: ButtonConfig {
            display_text: String::from("D"),
            keystroke: vec![vec![KeyboardCode::D]],
            ..Default::default()
        },
        button4: ButtonConfig {
            display_text: String::from("E"),
            keystroke: vec![vec![KeyboardCode::E]],
            ..Default::default()
        },
        button5: ButtonConfig {
            display_text: String::from("F"),
            keystroke: vec![vec![KeyboardCode::F]],
            ..Default::default()
        },
        button6: ButtonConfig {
            display_text: String::from("G"),
            keystroke: vec![vec![KeyboardCode::G]],
            ..Default::default()
        },
        button7: ButtonConfig {
            display_text: String::from("H"),
            keystroke: vec![vec![KeyboardCode::H]],
            ..Default::default()
        },
        button8: ButtonConfig {
            display_text: String::from("I"),
            keystroke: vec![vec![KeyboardCode::I]],
            ..Default::default()
        },
        button9: ButtonConfig {
            display_text: String::from("J"),
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
        menu_encoder: MenuEncoderConfig {
            display_text: String::from("Vol"),
            keystroke_left: vec![vec![KeyboardCode::VolumeDown]],
            keystroke_right: vec![vec![KeyboardCode::VolumeUp]],
            ..Default::default()
        },
        encoder1: EncoderConfig {
            display_text: String::from("Copy/Paste"),
            keystroke_left: vec![vec![KeyboardCode::LeftControl, KeyboardCode::C]],
            keystroke_right: vec![vec![KeyboardCode::LeftControl, KeyboardCode::V]],
            keystroke_push: vec![vec![KeyboardCode::LeftControl, KeyboardCode::V]],
            ..Default::default()
        },
        encoder2: EncoderConfig {
            display_text: String::from("Copy/Paste"),
            keystroke_left: vec![vec![KeyboardCode::LeftControl, KeyboardCode::C]],
            keystroke_right: vec![vec![KeyboardCode::LeftControl, KeyboardCode::V]],
            keystroke_push: vec![vec![KeyboardCode::LeftControl, KeyboardCode::V]],
            ..Default::default()
        },
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
