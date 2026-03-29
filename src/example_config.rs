use crate::*;
pub fn get_example_config() -> MacroConfig {
    MacroConfig {
        button0: ButtonConfig {
            display_text: String::from("A"),
            button: KeyboardCode::A,
        },
        button1: ButtonConfig {
            display_text: String::from("B"),
            button: KeyboardCode::B,
        },
        button2: ButtonConfig {
            display_text: String::from("C"),
            button: KeyboardCode::C,
        },
        button3: ButtonConfig {
            display_text: String::from("D"),
            button: KeyboardCode::D,
        },
        button4: ButtonConfig {
            display_text: String::from("E"),
            button: KeyboardCode::E,
        },
        button5: ButtonConfig {
            display_text: String::from("F"),
            button: KeyboardCode::F,
        },
        button6: ButtonConfig {
            display_text: String::from("G"),
            button: KeyboardCode::G,
        },
        button7: ButtonConfig {
            display_text: String::from("H"),
            button: KeyboardCode::H,
        },
        button8: ButtonConfig {
            display_text: String::from("I"),
            button: KeyboardCode::I,
        },
        button9: ButtonConfig {
            display_text: String::from("J"),
            button: KeyboardCode::J,
        },
        rotary_encoder1: RotaryEncoderConfig {
            left: ButtonConfig {
                display_text: String::from("V+"),
                button: KeyboardCode::VolumeUp,
            },
            right: ButtonConfig {
                display_text: String::from("V-"),
                button: KeyboardCode::VolumeDown,
            },
            push: ButtonConfig {
                display_text: String::from("Mute"),
                button: KeyboardCode::Mute,
            },
        },
        rotary_encoder2: RotaryEncoderConfig {
            left: ButtonConfig {
                display_text: String::from("V+"),
                button: KeyboardCode::VolumeUp,
            },
            right: ButtonConfig {
                display_text: String::from("V-"),
                button: KeyboardCode::VolumeDown,
            },
            push: ButtonConfig {
                display_text: String::from("Mute"),
                button: KeyboardCode::Mute,
            },
        },
        rotary_encover3: RotaryEncoderConfig {
            left: ButtonConfig {
                display_text: String::from("V+"),
                button: KeyboardCode::VolumeUp,
            },
            right: ButtonConfig {
                display_text: String::from("V-"),
                button: KeyboardCode::VolumeDown,
            },
            push: ButtonConfig {
                display_text: String::from("Mute"),
                button: KeyboardCode::Mute,
            },
        },
    }
}
