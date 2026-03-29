use crate::*;
pub fn get_example_config() -> MacroConfig {
    MacroConfig {
        name: "example".into(),
        button0: ButtonConfig {
            display_text: String::from("A"),
            key: KeyboardCode::A,
        },
        button1: ButtonConfig {
            display_text: String::from("B"),
            key: KeyboardCode::B,
        },
        button2: ButtonConfig {
            display_text: String::from("C"),
            key: KeyboardCode::C,
        },
        button3: ButtonConfig {
            display_text: String::from("D"),
            key: KeyboardCode::D,
        },
        button4: ButtonConfig {
            display_text: String::from("E"),
            key: KeyboardCode::E,
        },
        button5: ButtonConfig {
            display_text: String::from("F"),
            key: KeyboardCode::F,
        },
        button6: ButtonConfig {
            display_text: String::from("G"),
            key: KeyboardCode::G,
        },
        button7: ButtonConfig {
            display_text: String::from("H"),
            key: KeyboardCode::H,
        },
        button8: ButtonConfig {
            display_text: String::from("I"),
            key: KeyboardCode::I,
        },
        button9: ButtonConfig {
            display_text: String::from("J"),
            key: KeyboardCode::J,
        },
        menu_encoder: MenuEncoderConfig {
            display_text: String::from("Vol"),
            left: KeyboardCode::VolumeDown,
            right: KeyboardCode::VolumeUp,
        },
        encoder1: EncoderConfig {
            display_text: String::from("Vol"),
            left: KeyboardCode::VolumeDown,
            right: KeyboardCode::VolumeUp,
            push: KeyboardCode::Mute,
        },
        encoder2: EncoderConfig {
            display_text: String::from("Vol"),
            left: KeyboardCode::VolumeDown,
            right: KeyboardCode::VolumeUp,
            push: KeyboardCode::Mute,
        },
    }
}
