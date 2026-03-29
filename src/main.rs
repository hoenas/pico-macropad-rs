#![no_std]
#![no_main]

use panic_halt as _;

#[rtic::app(device = rp_pico::hal::pac, peripherals = true)]
mod app {

    extern crate alloc;
    use core::cell::RefCell;

    use alloc::string::String;
    use alloc::vec::Vec;
    use embedded_alloc::Heap;
    use embedded_graphics::primitives::Arc;
    use embedded_hal::digital::{InputPin, StatefulOutputPin};

    use embedded_menu::items::MenuItem;

    use fugit::MicrosDurationU32;
    use pico_macropad_rs::dummy_time_source::DummyTimesource;
    use pico_macropad_rs::read_config::write_last_config;
    use pico_macropad_rs::*;
    use rotary_encoder_hal::DefaultPhase;
    use rp_pico::XOSC_CRYSTAL_FREQ;
    // The macro for our start-up function

    // info!() and error!() macros for printing information to the debug output
    use defmt_rtt as _;

    // Ensure we halt the program on panic (if we don't mention this crate it won't
    // be linked)
    use panic_halt as _;

    use hal::gpio::Pin;

    use rp2040_hal::gpio::Interrupt::{EdgeHigh, EdgeLow};
    use rp2040_hal::gpio::{FunctionPio0, FunctionSpi, SioOutput};
    use rp2040_hal::Spi;

    use rp_pico::hal::clocks::init_clocks_and_plls;
    use rp_pico::hal::gpio::bank0::*;
    use rp_pico::hal::gpio::FunctionI2c;

    use rp_pico::hal::gpio::FunctionSio;
    use rp_pico::hal::gpio::FunctionSioOutput;
    use rp_pico::hal::gpio::PullDown;
    use rp_pico::hal::gpio::PullNone;
    use rp_pico::hal::gpio::PullUp;
    use rp_pico::hal::gpio::SioInput;
    use rp_pico::hal::Sio;
    use rp_pico::hal::Watchdog;
    use rp_pico::hal::I2C;
    use rp_pico::pac::{PIO0, SPI0};
    // Pull in any important traits
    use rp_pico::hal::prelude::*;

    // Embed the `Hz` function/trait:
    use fugit::RateExtU32;
    // Import the SPI abstraction:
    use rp_pico::hal::spi;

    // Import the GPIO abstraction:
    use rp_pico::hal::gpio;

    // A shorter alias for the Hardware Abstraction Layer, which provides
    // higher-level drivers.
    use rp_pico::hal;

    // Link in the embedded_sdmmc crate.
    // The `SdMmcSpi` is used for block level access to the card.
    // And the `VolumeManager` gives access to the FAT filesystem functions.
    use embedded_sdmmc::{SdCard, VolumeManager};

    use embedded_hal::delay::DelayNs;
    use embedded_hal::digital::OutputPin;
    use rp_pico::hal::Timer;

    use embedded_graphics::{
        mono_font::{ascii::FONT_6X10, MonoTextStyle},
        pixelcolor::BinaryColor,
        prelude::*,
        text::{Alignment, Text},
    };
    use rotary_encoder_hal::{Direction, Rotary};
    use rp2040_hal::pio::SM0;
    use rp2040_hal::spi::Enabled;
    use rp2040_hal::timer::CountDown;
    use rp_pico::hal::timer::Alarm;
    use rp_pico::pac::I2C1;
    use sh1106::{prelude::*, Builder};
    use smart_leds::SmartLedsWrite;
    use smart_leds::RGB8;
    use ws2812_pio::Ws2812;
    // USB Human Interface Device (HID) Class support
    use embedded_hal_bus::spi::{ExclusiveDevice, NoDelay};

    const DISPLAY_UPDATE: MicrosDurationU32 = MicrosDurationU32::millis(25);
    const RGB_LEDS_UPDATE: MicrosDurationU32 = MicrosDurationU32::millis(25);
    const NUM_LEDS: usize = 8;
    const CHARACTER_STYLE: MonoTextStyle<BinaryColor> =
        MonoTextStyle::new(&FONT_6X10, BinaryColor::On);

    pub struct Encoder {
        pub value: usize,
        pub value_changed: bool,
        pub delta: i8,
        pub button: bool,
    }
    pub struct Encoders {
        pub encoder1: Encoder,
        pub encoder2: Encoder,
        pub encoder3: Encoder,
    }
    impl Default for Encoders {
        fn default() -> Self {
            Self {
                encoder1: Encoder {
                    value: 0,
                    value_changed: false,
                    delta: 0,
                    button: false,
                },
                encoder2: Encoder {
                    value: 0,
                    value_changed: false,
                    delta: 0,
                    button: false,
                },
                encoder3: Encoder {
                    value: 0,
                    value_changed: false,
                    delta: 0,
                    button: false,
                },
            }
        }
    }

    #[shared]
    struct Shared {
        timer: hal::Timer,
        led: Pin<Gpio25, FunctionSioOutput, PullNone>,
        encoders: Encoders,
        config: MacroConfig,
    }

    #[local]
    struct Local {
        display_alarm: hal::timer::Alarm0,
        rgb_leds_alarm: hal::timer::Alarm1,
        rotary_encoder1: Rotary<
            Pin<Gpio10, FunctionSio<SioInput>, PullNone>,
            Pin<Gpio11, FunctionSio<SioInput>, PullNone>,
            DefaultPhase,
        >,
        rotary_encoder1_switch: Pin<Gpio12, FunctionSio<SioInput>, PullUp>,
        rotary_encoder2: Rotary<
            Pin<Gpio13, FunctionSio<SioInput>, PullNone>,
            Pin<Gpio14, FunctionSio<SioInput>, PullNone>,
            DefaultPhase,
        >,
        rotary_encoder2_switch: Pin<Gpio15, FunctionSio<SioInput>, PullUp>,
        rotary_encoder3: Rotary<
            Pin<Gpio20, FunctionSio<SioInput>, PullNone>,
            Pin<Gpio21, FunctionSio<SioInput>, PullNone>,
            DefaultPhase,
        >,
        rotary_encoder3_switch: Pin<Gpio22, FunctionSio<SioInput>, PullUp>,
        display: GraphicsMode<
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
        rgb_leds: Ws2812<PIO0, SM0, CountDown, Pin<Gpio28, FunctionPio0, PullDown>>,
        menu: embedded_menu::Menu<
            &'static str,
            embedded_menu::interaction::programmed::Programmed,
            embedded_layout::prelude::Chain<
                embedded_menu::collection::MenuItems<
                    Vec<MenuItem<String, (), &'static str, true>>,
                    MenuItem<String, (), &'static str, true>,
                    (),
                >,
            >,
            (),
            embedded_menu::selection_indicator::StaticPosition,
            embedded_menu::selection_indicator::style::Line,
            BinaryColor,
        >,
        file_names: Vec<String>,
        menu_mode: bool,
        ticks_since_menu_state_change: usize,
        display_update_interval: usize,
        sd_volume_mgr: VolumeManager<
            SdCard<
                ExclusiveDevice<
                    Spi<
                        Enabled,
                        SPI0,
                        (
                            Pin<Gpio19, FunctionSpi, PullNone>,
                            Pin<Gpio16, FunctionSpi, PullUp>,
                            Pin<Gpio18, FunctionSpi, PullNone>,
                        ),
                    >,
                    Pin<Gpio17, FunctionSio<SioOutput>, PullDown>,
                    NoDelay,
                >,
                Timer,
            >,
            DummyTimesource,
        >,
    }

    fn check_state_changed(interval: usize, counter: usize) -> bool {
        counter as f32 / interval as f32 > 0.75
    }

    #[init]
    fn init(c: init::Context) -> (Shared, Local, init::Monotonics) {
        #[global_allocator]
        static ALLOCATOR: Heap = Heap::empty();
        {
            use core::mem::MaybeUninit;
            const HEAP_SIZE: usize = 10240;
            static mut HEAP: [MaybeUninit<u8>; HEAP_SIZE] = [MaybeUninit::uninit(); HEAP_SIZE];
            unsafe { ALLOCATOR.init(core::ptr::addr_of_mut!(HEAP) as usize, HEAP_SIZE) }
        }
        // Soft-reset does not release the hardware spinlocks
        // Release them now to avoid a deadlock after debug or watchdog reset
        unsafe {
            hal::sio::spinlock_reset();
        }
        let mut resets = c.device.RESETS;
        let mut watchdog = Watchdog::new(c.device.WATCHDOG);
        let clocks = init_clocks_and_plls(
            XOSC_CRYSTAL_FREQ,
            c.device.XOSC,
            c.device.CLOCKS,
            c.device.PLL_SYS,
            c.device.PLL_USB,
            &mut resets,
            &mut watchdog,
        )
        .ok()
        .unwrap();

        let sio = Sio::new(c.device.SIO);
        let pins = rp_pico::Pins::new(
            c.device.IO_BANK0,
            c.device.PADS_BANK0,
            sio.gpio_bank0,
            &mut resets,
        );
        // Pin setup
        // Onboard LED
        let mut led = pins.led.reconfigure();
        led.is_set_low().unwrap();
        // Buttons
        let _button0 = pins.gpio0.into_pull_up_input();
        let _button1 = pins.gpio1.into_pull_up_input();
        let _button2 = pins.gpio2.into_pull_up_input();
        let _button3 = pins.gpio3.into_pull_up_input();
        let _button4 = pins.gpio4.into_pull_up_input();
        let _button5 = pins.gpio5.into_pull_up_input();
        let _button6 = pins.gpio6.into_pull_up_input();
        let _button7 = pins.gpio7.into_pull_up_input();
        let _button8 = pins.gpio8.into_pull_up_input();
        let _button9 = pins.gpio9.into_pull_up_input();
        // Rotary encoders
        // - Encoder 1
        let gpio10 = pins.gpio10.into_floating_input();
        gpio10.set_interrupt_enabled(EdgeHigh, true);
        gpio10.set_interrupt_enabled(EdgeLow, true);
        let gpio11 = pins.gpio11.into_floating_input();
        gpio11.set_interrupt_enabled(EdgeHigh, true);
        gpio11.set_interrupt_enabled(EdgeLow, true);
        let rotary_encoder1 = Rotary::new(gpio10, gpio11);
        let rotary_encoder1_switch = pins.gpio12.into_pull_up_input();
        rotary_encoder1_switch.set_interrupt_enabled(EdgeLow, true);
        // - Encoder 2
        let gpio13 = pins.gpio13.into_floating_input();
        gpio13.set_interrupt_enabled(EdgeLow, true);
        gpio13.set_interrupt_enabled(EdgeHigh, true);
        let gpio14 = pins.gpio14.into_floating_input();
        gpio14.set_interrupt_enabled(EdgeHigh, true);
        gpio14.set_interrupt_enabled(EdgeLow, true);
        let rotary_encoder2 = Rotary::new(gpio13, gpio14);
        let rotary_encoder2_switch = pins.gpio15.into_pull_up_input();
        rotary_encoder2_switch.set_interrupt_enabled(EdgeLow, true);
        // - Encoder 3
        let gpio20 = pins.gpio20.into_floating_input();
        gpio20.set_interrupt_enabled(EdgeLow, true);
        gpio20.set_interrupt_enabled(EdgeHigh, true);
        let gpio21 = pins.gpio21.into_floating_input();
        gpio21.set_interrupt_enabled(EdgeHigh, true);
        gpio21.set_interrupt_enabled(EdgeLow, true);
        let rotary_encoder3 = Rotary::new(gpio20, gpio21);
        let rotary_encoder3_switch = pins.gpio22.into_pull_up_input();
        rotary_encoder3_switch.set_interrupt_enabled(EdgeLow, true);
        // Display
        let display_sda_pin: hal::gpio::Pin<_, hal::gpio::FunctionI2C, _> =
            pins.gpio26.reconfigure();
        let display_scl_pin: hal::gpio::Pin<_, hal::gpio::FunctionI2C, _> =
            pins.gpio27.reconfigure();
        let display_i2c = hal::I2C::i2c1(
            c.device.I2C1,
            display_sda_pin,
            display_scl_pin,
            400.kHz(),
            &mut resets,
            &clocks.peripheral_clock,
        );
        let mut display: GraphicsMode<_> = Builder::new().connect_i2c(display_i2c).into();
        display.init().unwrap();
        display.clear();
        display.flush().unwrap();

        // SDCard
        // - Set up our SPI pins into the correct mode
        let sdmmc_spi_sclk: gpio::Pin<_, gpio::FunctionSpi, gpio::PullNone> =
            pins.gpio18.reconfigure();
        let sdmmc_spi_mosi: gpio::Pin<_, gpio::FunctionSpi, gpio::PullNone> =
            pins.gpio19.reconfigure();
        let sdmmc_spi_miso: gpio::Pin<_, gpio::FunctionSpi, gpio::PullUp> =
            pins.gpio16.reconfigure();
        let sdmmc_spi_cs = pins.gpio17.into_push_pull_output();
        // - Create the SPI driver instance for the SPI0 device
        let spi0 = spi::Spi::<_, _, _, 8>::new(
            c.device.SPI0,
            (sdmmc_spi_mosi, sdmmc_spi_miso, sdmmc_spi_sclk),
        );

        // - Exchange the uninitialised SPI driver for an initialised one
        let sdmmc_spi_bus = spi0.init(
            &mut resets,
            clocks.peripheral_clock.freq(),
            400.kHz(), // card initialization happens at low baud rate
            embedded_hal::spi::MODE_0,
        );
        let sdmmc_spi = ExclusiveDevice::new_no_delay(sdmmc_spi_bus, sdmmc_spi_cs).unwrap();

        display.clear();
        display.flush().unwrap();

        let mut timer = Timer::new(c.device.TIMER, &mut resets, &clocks);
        let sdcard = SdCard::new(sdmmc_spi, timer);
        let volume_mgr = embedded_sdmmc::VolumeManager::new(
            sdcard,
            dummy_time_source::DummyTimesource::default(),
        );
        Text::with_alignment(
            "Opening SDCard...",
            display.bounding_box().top_left + Point::new(0, 10),
            CHARACTER_STYLE,
            Alignment::Left,
        )
        .draw(&mut display)
        .unwrap();
        display.flush().unwrap();
        let volume0 = volume_mgr
            .open_volume(embedded_sdmmc::VolumeIdx(0))
            .unwrap();
        Text::with_alignment(
            "Listing files...",
            display.bounding_box().top_left + Point::new(0, 20),
            CHARACTER_STYLE,
            Alignment::Left,
        )
        .draw(&mut display)
        .unwrap();
        display.flush().unwrap();

        let root_dir = volume0.open_root_dir().unwrap();
        let mut buffer = [0_u8; 4096];
        let mut lfn_buffer = embedded_sdmmc::LfnBuffer::new(&mut buffer);
        let mut menu_items = Vec::new();
        let mut file_names = Vec::new();
        root_dir
            .iterate_dir_lfn(&mut lfn_buffer, |_, filename| {
                if let Some(filename) = filename {
                    if filename.ends_with(".cfg") {
                        let item = MenuItem::new(String::from(filename), "");
                        file_names.push(String::from(filename));
                        menu_items.push(item);
                    }
                }
            })
            .unwrap();
        let last_config = read_config::get_last_config(&root_dir).unwrap_or(
            file_names
                .first()
                .cloned()
                .unwrap_or(String::from("No config found")),
        );
        Text::with_alignment(
            &alloc::format!("Loading last config:\n{}", last_config),
            display.bounding_box().top_left + Point::new(0, 30),
            CHARACTER_STYLE,
            Alignment::Left,
        )
        .draw(&mut display)
        .unwrap();
        display.flush().unwrap();

        let config = read_config::read_config_file(&root_dir, last_config.as_str()).unwrap();
        drop(root_dir);
        drop(volume0);
        let style = embedded_menu::MenuStyle::new(BinaryColor::On)
            .with_scrollbar_style(embedded_menu::DisplayScrollbar::Auto);
        let menu = embedded_menu::Menu::with_style("Load Config", style)
            .add_menu_items(menu_items)
            .build();
        // - RGB LEDs
        let (mut pio, sm0, _, _, _) = c.device.PIO0.split(&mut resets);

        let rgb_leds = Ws2812::new(
            pins.gpio28.into_function(),
            &mut pio,
            sm0,
            clocks.peripheral_clock.freq(),
            timer.count_down(),
        );

        // Timer for display update
        let mut display_alarm = timer.alarm_0().unwrap();
        let _ = display_alarm.schedule(DISPLAY_UPDATE);
        display_alarm.enable_interrupt();
        // Timer for RGB LED update
        let mut rgb_leds_alarm = timer.alarm_1().unwrap();
        let _ = rgb_leds_alarm.schedule(RGB_LEDS_UPDATE);
        rgb_leds_alarm.enable_interrupt();

        (
            Shared {
                timer,
                led,
                encoders: Encoders::default(),
                config,
            },
            Local {
                display_alarm,
                rgb_leds_alarm,
                rotary_encoder1,
                rotary_encoder1_switch,
                rotary_encoder2,
                rotary_encoder2_switch,
                rotary_encoder3,
                rotary_encoder3_switch,
                display,
                rgb_leds,
                menu,
                file_names,
                menu_mode: false,
                ticks_since_menu_state_change: 0,
                display_update_interval: DISPLAY_UPDATE.to_millis() as usize,
                sd_volume_mgr: volume_mgr,
            },
            init::Monotonics(),
        )
    }

    #[task(
        binds = TIMER_IRQ_0,
        priority = 3,
        shared = [timer, led, encoders, config],
        local = [tog: bool = true, display, menu, menu_mode, ticks_since_menu_state_change, display_update_interval, file_names,display_alarm, sd_volume_mgr],
    )]
    fn display_update(mut c: display_update::Context) {
        c.local.display.clear();
        c.shared.led.lock(|l| l.set_high().unwrap());
        // Check if we are in menu mode
        let menu_mode = *c.local.menu_mode;
        let mut menu_button_pressed = false;
        let mut menu_button_state_changed = false;
        c.shared.encoders.lock(|encoders| {
            menu_button_pressed = encoders.encoder1.button;
            menu_button_state_changed = encoders.encoder1.value_changed;
        });
        // Switch to menu mode if we are currently not in menu mode and encoder1 button is pressed
        if !menu_mode && menu_button_pressed && menu_button_state_changed {
            *c.local.menu_mode = true;
        }
        // Update menu
        if menu_mode {
            let menu = c.local.menu;
            // Check if the encoder has been rotated
            let mut menu_position = 0;
            c.shared.encoders.lock(|encoders| {
                menu_position = encoders.encoder1.value;
                if menu_position >= c.local.file_names.len() {
                    menu_position = c.local.file_names.len() - 1;
                    encoders.encoder1.value = menu_position;
                }
            });

            menu.interact(embedded_menu::interaction::Interaction::Navigation(
                embedded_menu::interaction::Navigation::JumpTo(menu_position),
            ));
            if menu_button_pressed && menu_button_state_changed {
                // Load config
                let volume = c
                    .local
                    .sd_volume_mgr
                    .open_volume(embedded_sdmmc::VolumeIdx(0))
                    .unwrap();
                let root_dir = volume.open_root_dir().unwrap();
                let selected_file_name = c.local.file_names[menu_position].clone();
                if let Ok(new_config) =
                    read_config::read_config_file(&root_dir, selected_file_name.as_str())
                {
                    c.shared.config.lock(|config| *config = new_config);
                    write_last_config(&root_dir, &selected_file_name.as_str());
                    *c.local.menu_mode = false;
                }
            }
            menu.update(c.local.display);
            menu.draw(c.local.display);
        } else {
            // TODO: Display key map related stuff
        }
        c.local.display.flush().unwrap();
        c.local.display_alarm.clear_interrupt();
        c.local.display_alarm.schedule(DISPLAY_UPDATE).unwrap();
    }

    #[task(
        binds = TIMER_IRQ_1,
        priority = 4,
        shared = [timer ],
        local = [tog: bool = true, animation_counter: usize = 0, rgb_leds, rgb_leds_alarm],
    )]
    fn leds_update(c: leds_update::Context) {
        *c.local.animation_counter += 1;
        let _counter = c.local.animation_counter;

        // Write RGB values
        let mut data: [RGB8; NUM_LEDS] = [RGB8::default(); NUM_LEDS];
        for (i, led) in data.iter_mut().enumerate() {
            let red = 30u8 * (i as u8 + 1_u8);
            let blue = 255u8 - 30u8 * (i as u8 + 1_u8);
            *led = RGB8::new(red, 0, blue);
        }
        c.local.rgb_leds.write(data.iter().cloned()).unwrap();

        c.local.rgb_leds_alarm.clear_interrupt();
        c.local.rgb_leds_alarm.schedule(RGB_LEDS_UPDATE).unwrap();
    }

    #[task(
        binds = IO_IRQ_BANK0,
        priority = 1,
        shared = [led, encoders, timer],
        local = [tog: bool = true, rotary_encoder1, rotary_encoder1_switch, rotary_encoder2, rotary_encoder2_switch, rotary_encoder3, rotary_encoder3_switch],
    )]
    fn rotary_encoder_update(mut c: rotary_encoder_update::Context) {
        c.shared.led.lock(|l| l.set_low().unwrap());

        *c.local.tog = !*c.local.tog;

        // Check encoders
        // - Encoder1
        let encoder1_increment = if let Ok(direction) = c.local.rotary_encoder1.update() {
            if direction == Direction::Clockwise {
                -1
            } else if direction == Direction::CounterClockwise {
                1
            } else {
                0
            }
        } else {
            0
        };
        let encoder1_switch_value = c.local.rotary_encoder1_switch.is_low().unwrap();

        // - Encoder2
        let encoder2_increment = if let Ok(direction) = c.local.rotary_encoder2.update() {
            if direction == Direction::Clockwise {
                -1
            } else if direction == Direction::CounterClockwise {
                1
            } else {
                0
            }
        } else {
            0
        };
        let encoder2_switch_value = c.local.rotary_encoder2_switch.is_low().unwrap();
        // - Encoder3
        let encoder3_increment = if let Ok(direction) = c.local.rotary_encoder3.update() {
            if direction == Direction::Clockwise {
                -1
            } else if direction == Direction::CounterClockwise {
                1
            } else {
                0
            }
        } else {
            0
        };
        let encoder3_switch_value = c.local.rotary_encoder3_switch.is_low().unwrap();

        // Write values

        (c.shared.encoders, c.shared.timer).lock(|encoders, timer| {
            // - Encoder1
            let encoder_1_value = encoders.encoder1.value as i32 + encoder1_increment;
            encoders.encoder1.value = if encoder_1_value < 0 {
                0
            } else {
                encoder_1_value.try_into().unwrap()
            };
            encoders.encoder1.value_changed = encoders.encoder1.button != encoder1_switch_value;
            encoders.encoder1.delta = encoder1_increment.try_into().unwrap();
            encoders.encoder1.button = encoder1_switch_value;
            // - Encoder2
            let encoder_2_value = encoders.encoder2.value as i32 + encoder2_increment;
            encoders.encoder2.value = if encoder_2_value < 0 {
                0
            } else {
                encoder_2_value.try_into().unwrap()
            };
            encoders.encoder2.value_changed = encoders.encoder2.button != encoder2_switch_value;
            encoders.encoder2.delta = encoder2_increment.try_into().unwrap();
            encoders.encoder2.button = encoder2_switch_value;
            // - Encoder3
            let encoder_3_value = encoders.encoder3.value as i32 + encoder3_increment;
            encoders.encoder3.value = if encoder_3_value < 0 {
                0
            } else {
                encoder_3_value.try_into().unwrap()
            };
            encoders.encoder3.value_changed = encoders.encoder3.button != encoder3_switch_value;
            encoders.encoder3.delta = encoder3_increment.try_into().unwrap();
            encoders.encoder3.button = encoder3_switch_value;
            // Debounce push buttons
            if encoders.encoder1.value_changed
                || encoders.encoder2.value_changed
                || encoders.encoder3.value_changed
            {
                timer.delay_ms(25u32);
            }
            // Debounce rotary encoders
            if encoders.encoder1.delta != 0
                || encoders.encoder2.delta != 0
                || encoders.encoder3.delta != 0
            {
                timer.delay_us(100u32);
            }
        });
    }
}
