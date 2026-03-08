use crate::*;
pub trait TestConfig {
    fn get_test_config() -> Self;
}
impl TestConfig for MacroConfig {
    fn get_test_config() -> MacroConfig {
        MacroConfig {
            button0: ButtonConfig {
                display_text: String::from("A"),
                button: KeyboardCode::A,
            },
            button1: ButtonConfig {
                display_text: String::from("A"),
                button: KeyboardCode::A,
            },
            button2: ButtonConfig {
                display_text: String::from("A"),
                button: KeyboardCode::A,
            },
            button3: ButtonConfig {
                display_text: String::from("A"),
                button: KeyboardCode::A,
            },
            button4: ButtonConfig {
                display_text: String::from("A"),
                button: KeyboardCode::A,
            },
            button5: ButtonConfig {
                display_text: String::from("A"),
                button: KeyboardCode::A,
            },
            button6: ButtonConfig {
                display_text: String::from("A"),
                button: KeyboardCode::A,
            },
            button7: ButtonConfig {
                display_text: String::from("A"),
                button: KeyboardCode::A,
            },
            button8: ButtonConfig {
                display_text: String::from("A"),
                button: KeyboardCode::A,
            },
            button9: ButtonConfig {
                display_text: String::from("A"),
                button: KeyboardCode::A,
            },
            rotary_encoder1: RotaryEncoderConfig {
                display_text: String::from("A"),
                left: ButtonConfig {
                    display_text: String::from("A"),
                    button: KeyboardCode::A,
                },
                right: ButtonConfig {
                    display_text: String::from("A"),
                    button: KeyboardCode::A,
                },
                push: ButtonConfig {
                    display_text: String::from("A"),
                    button: KeyboardCode::A,
                },
            },
            rotary_encoder2: RotaryEncoderConfig {
                display_text: String::from("A"),
                left: ButtonConfig {
                    display_text: String::from("A"),
                    button: KeyboardCode::A,
                },
                right: ButtonConfig {
                    display_text: String::from("A"),
                    button: KeyboardCode::A,
                },
                push: ButtonConfig {
                    display_text: String::from("A"),
                    button: KeyboardCode::A,
                },
            },
            rotary_encover3: RotaryEncoderConfig {
                display_text: String::from("A"),
                left: ButtonConfig {
                    display_text: String::from("A"),
                    button: KeyboardCode::A,
                },
                right: ButtonConfig {
                    display_text: String::from("A"),
                    button: KeyboardCode::A,
                },
                push: ButtonConfig {
                    display_text: String::from("A"),
                    button: KeyboardCode::A,
                },
            },
        }
    }
}
