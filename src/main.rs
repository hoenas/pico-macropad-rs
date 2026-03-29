#![no_std]
#![no_main]

use panic_halt as _;

#[rtic::app(device = rp_pico::hal::pac, peripherals = true)]
mod app {

    extern crate alloc;

    use alloc::string::String;
    use alloc::vec::Vec;
    use embedded_alloc::Heap;
    use embedded_hal::digital::{InputPin, StatefulOutputPin};

    use embedded_menu::items::MenuItem;

    use fugit::MicrosDurationU32;
    use pico_macropad_rs::containers::{Encoders, MacroPadButtons};
    use pico_macropad_rs::dummy_time_source::DummyTimesource;
    use pico_macropad_rs::read_config::{write_example_config_file, write_last_config};
    use pico_macropad_rs::update_display::CHARACTER_STYLE;
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
        pixelcolor::BinaryColor,
        prelude::*,
        text::{Alignment, Text},
    };
    use frunk::HList;
    use rotary_encoder_hal::{Direction, Rotary};
    use rp2040_hal::pio::SM0;
    use rp2040_hal::spi::Enabled;
    use rp2040_hal::timer::CountDown;
    use rp_pico::hal::timer::Alarm;
    use rp_pico::pac::I2C1;
    use sh1106::{prelude::*, Builder};
    use smart_leds::SmartLedsWrite;
    use smart_leds::RGB8;
    use usb_device::bus::UsbBusAllocator;
    use usb_device::device::{StringDescriptors, UsbDevice, UsbDeviceBuilder, UsbVidPid};
    use usbd_hid::UsbError;
    use usbd_human_interface_device::device::keyboard::NKROBootKeyboard;
    use usbd_human_interface_device::prelude::{UsbHidClass, UsbHidClassBuilder};
    use usbd_human_interface_device::UsbHidError;
    use ws2812_pio::Ws2812;
    // USB Human Interface Device (HID) Class support
    use embedded_hal_bus::spi::{ExclusiveDevice, NoDelay};
    use usbd_human_interface_device::page::Keyboard;

    const DISPLAY_UPDATE: MicrosDurationU32 = MicrosDurationU32::millis(25);
    const KEYBOARD_UPDATE_MILIS: usize = 1;
    const KEYBOARD_UPDATE: MicrosDurationU32 =
        MicrosDurationU32::millis(KEYBOARD_UPDATE_MILIS as u32);
    const KEYBOARD_KEY_CHECK_INTERVAL: usize = 50 / KEYBOARD_UPDATE_MILIS;
    const NUM_LEDS: usize = 8;

    #[shared]
    struct Shared {
        timer: hal::Timer,
        led: Pin<Gpio25, FunctionSioOutput, PullNone>,
        encoders: Encoders,
        buttons: MacroPadButtons,
        config: MacroConfig,
        menu_mode: bool,
        keyboard: UsbHidClass<
            'static,
            hal::usb::UsbBus,
            HList!(NKROBootKeyboard<'static, hal::usb::UsbBus>),
        >,
    }

    #[local]
    struct Local {
        display_alarm: hal::timer::Alarm0,
        keyboard_tick_alarm: hal::timer::Alarm1,
        menu_encoder: Rotary<
            Pin<Gpio10, FunctionSio<SioInput>, PullNone>,
            Pin<Gpio11, FunctionSio<SioInput>, PullNone>,
            DefaultPhase,
        >,
        menu_encoder_switch: Pin<Gpio12, FunctionSio<SioInput>, PullUp>,
        encoder1: Rotary<
            Pin<Gpio13, FunctionSio<SioInput>, PullNone>,
            Pin<Gpio14, FunctionSio<SioInput>, PullNone>,
            DefaultPhase,
        >,
        encoder1_switch: Pin<Gpio15, FunctionSio<SioInput>, PullUp>,
        encoder2: Rotary<
            Pin<Gpio20, FunctionSio<SioInput>, PullNone>,
            Pin<Gpio21, FunctionSio<SioInput>, PullNone>,
            DefaultPhase,
        >,
        encoder2_switch: Pin<Gpio22, FunctionSio<SioInput>, PullUp>,
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
        button0: Pin<Gpio0, FunctionSio<SioInput>, PullUp>,
        button1: Pin<Gpio1, FunctionSio<SioInput>, PullUp>,
        button2: Pin<Gpio2, FunctionSio<SioInput>, PullUp>,
        button3: Pin<Gpio3, FunctionSio<SioInput>, PullUp>,
        button4: Pin<Gpio4, FunctionSio<SioInput>, PullUp>,
        button5: Pin<Gpio5, FunctionSio<SioInput>, PullUp>,
        button6: Pin<Gpio6, FunctionSio<SioInput>, PullUp>,
        button7: Pin<Gpio7, FunctionSio<SioInput>, PullUp>,
        button8: Pin<Gpio8, FunctionSio<SioInput>, PullUp>,
        button9: Pin<Gpio9, FunctionSio<SioInput>, PullUp>,
        usb_device: UsbDevice<'static, hal::usb::UsbBus>,
    }

    fn check_state_changed(interval: usize, counter: usize) -> bool {
        counter as f32 / interval as f32 > 0.75
    }

    #[init(local = [usb_alloc: Option<UsbBusAllocator<hal::usb::UsbBus>> = None])]
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
        let clocks: rp2040_hal::clocks::ClocksManager = init_clocks_and_plls(
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
        let button0 = pins.gpio0.into_pull_up_input();
        let button1 = pins.gpio1.into_pull_up_input();
        let button2 = pins.gpio2.into_pull_up_input();
        let button3 = pins.gpio3.into_pull_up_input();
        let button4 = pins.gpio4.into_pull_up_input();
        let button5 = pins.gpio5.into_pull_up_input();
        let button6 = pins.gpio6.into_pull_up_input();
        let button7 = pins.gpio7.into_pull_up_input();
        let button8 = pins.gpio8.into_pull_up_input();
        let button9 = pins.gpio9.into_pull_up_input();
        // Rotary encoders
        // - Menu encoder
        let gpio10 = pins.gpio10.into_floating_input();
        gpio10.set_interrupt_enabled(EdgeHigh, true);
        gpio10.set_interrupt_enabled(EdgeLow, true);
        let gpio11 = pins.gpio11.into_floating_input();
        gpio11.set_interrupt_enabled(EdgeHigh, true);
        gpio11.set_interrupt_enabled(EdgeLow, true);
        let menu_encoder = Rotary::new(gpio10, gpio11);
        let menu_encoder_switch = pins.gpio12.into_pull_up_input();
        menu_encoder_switch.set_interrupt_enabled(EdgeLow, true);
        // - Encoder 1
        let gpio13 = pins.gpio13.into_floating_input();
        gpio13.set_interrupt_enabled(EdgeLow, true);
        gpio13.set_interrupt_enabled(EdgeHigh, true);
        let gpio14 = pins.gpio14.into_floating_input();
        gpio14.set_interrupt_enabled(EdgeHigh, true);
        gpio14.set_interrupt_enabled(EdgeLow, true);
        let encoder1 = Rotary::new(gpio13, gpio14);
        let encoder1_switch = pins.gpio15.into_pull_up_input();
        encoder1_switch.set_interrupt_enabled(EdgeLow, true);
        // - Encoder 2
        let gpio20 = pins.gpio20.into_floating_input();
        gpio20.set_interrupt_enabled(EdgeLow, true);
        gpio20.set_interrupt_enabled(EdgeHigh, true);
        let gpio21 = pins.gpio21.into_floating_input();
        gpio21.set_interrupt_enabled(EdgeHigh, true);
        gpio21.set_interrupt_enabled(EdgeLow, true);
        let encoder2 = Rotary::new(gpio20, gpio21);
        let encoder2_switch = pins.gpio22.into_pull_up_input();
        encoder2_switch.set_interrupt_enabled(EdgeLow, true);
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
        write_example_config_file(&root_dir, &"good.cfg");
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
        // - USB
        let usb_alloc = c
            .local
            .usb_alloc
            .insert(UsbBusAllocator::new(hal::usb::UsbBus::new(
                c.device.USBCTRL_REGS,
                c.device.USBCTRL_DPRAM,
                clocks.usb_clock,
                true,
                &mut resets,
            )));

        let keyboard = UsbHidClassBuilder::new()
            .add_device(
                usbd_human_interface_device::device::keyboard::NKROBootKeyboardConfig::default(),
            )
            .build(usb_alloc);

        // https://pid.codes
        let usb_dev = UsbDeviceBuilder::new(usb_alloc, UsbVidPid(0x1209, 0x0001))
            .strings(&[StringDescriptors::default()
                .manufacturer("usbd-human-interface-device")
                .product("Keyboard")
                .serial_number("TEST")])
            .unwrap()
            .build();

        // Enable the USB interrupt
        unsafe {
            rp_pico::pac::NVIC::unmask(hal::pac::Interrupt::USBCTRL_IRQ);
        };
        // Timer for display update
        let mut display_alarm = timer.alarm_0().unwrap();
        let _ = display_alarm.schedule(DISPLAY_UPDATE);
        display_alarm.enable_interrupt();
        // Timer for RGB LED update
        let mut keyboard_tick_alarm = timer.alarm_1().unwrap();
        let _ = keyboard_tick_alarm.schedule(KEYBOARD_UPDATE);
        keyboard_tick_alarm.enable_interrupt();

        (
            Shared {
                timer,
                led,
                encoders: Encoders::default(),
                config,
                buttons: MacroPadButtons::default(),
                menu_mode: false,
                keyboard,
            },
            Local {
                display_alarm,
                keyboard_tick_alarm,
                menu_encoder,
                menu_encoder_switch,
                encoder1,
                encoder1_switch,
                encoder2,
                encoder2_switch,
                display,
                rgb_leds,
                menu,
                file_names,
                ticks_since_menu_state_change: 0,
                display_update_interval: DISPLAY_UPDATE.to_millis() as usize,
                sd_volume_mgr: volume_mgr,
                button0,
                button1,
                button2,
                button3,
                button4,
                button5,
                button6,
                button7,
                button8,
                button9,
                usb_device: usb_dev,
            },
            init::Monotonics(),
        )
    }

    #[task(
        binds = TIMER_IRQ_0,
        shared = [timer, encoders, config, menu_mode, led],
        local = [display, menu, ticks_since_menu_state_change, display_update_interval, file_names,display_alarm, sd_volume_mgr, rgb_leds],
    )]
    fn display_update(mut c: display_update::Context) {
        c.shared.led.lock(|l| l.set_high().unwrap());
        c.local.display.clear();
        // Check if we are in menu mode
        let mut menu_mode = false;
        c.shared.menu_mode.lock(|mode| menu_mode = *mode);
        let mut menu_button_pressed = false;
        let mut menu_button_state_changed = false;
        c.shared.encoders.lock(|encoders| {
            menu_button_pressed = encoders.menu_encoder.button;
            menu_button_state_changed = encoders.menu_encoder.value_changed;
        });
        // Switch to menu mode if we are currently not in menu mode and encoder0 button is pressed
        if !menu_mode && menu_button_pressed && menu_button_state_changed {
            c.shared.menu_mode.lock(|menu_mode| *menu_mode = true);
        }
        // Update menu
        if menu_mode {
            let menu = c.local.menu;
            // Check if the encoder has been rotated
            let mut menu_position = 0;
            c.shared.encoders.lock(|encoders| {
                menu_position = encoders.menu_encoder.value;
                if menu_position >= c.local.file_names.len() {
                    menu_position = c.local.file_names.len() - 1;
                    encoders.menu_encoder.value = menu_position;
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
                    c.shared.menu_mode.lock(|menu_mode| *menu_mode = false);
                }
            }
            menu.update(c.local.display);
            menu.draw(c.local.display);
        } else {
            // TODO: Do display stuff
            // Update LEDs
            // Write RGB values
            let mut data: [RGB8; NUM_LEDS] = [RGB8::default(); NUM_LEDS];
            for (i, led) in data.iter_mut().enumerate() {
                let red = 30u8 * (i as u8 + 1_u8);
                let blue = 255u8 - 30u8 * (i as u8 + 1_u8);
                *led = RGB8::new(red, 0, blue);
            }
            c.local.rgb_leds.write(data.iter().cloned()).unwrap();
        }
        c.local.display.flush().unwrap();
        c.local.display_alarm.clear_interrupt();
        c.local.display_alarm.schedule(DISPLAY_UPDATE).unwrap();
    }

    #[task(
        binds = TIMER_IRQ_1,
        priority = 2,
        shared = [timer,keyboard, encoders, buttons, config, menu_mode, led],
        local = [keyboard_tick_alarm, ticks_since_key_check: usize = 0],
    )]
    fn keyboard_tick(mut c: keyboard_tick::Context) {
        c.shared.led.lock(|l| l.set_low().unwrap());
        c.shared.keyboard.lock(|k| match k.tick() {
            Err(UsbHidError::WouldBlock) => {}
            Ok(_) => {}
            Err(e) => {
                core::panic!("Failed to process keyboard tick: {:?}", e)
            }
        });

        let mut menu_mode = false;
        c.shared.menu_mode.lock(|mode| menu_mode = *mode);
        // Skip writing keyboard reports if we are in menu mode

        if *c.local.ticks_since_key_check > KEYBOARD_KEY_CHECK_INTERVAL {
            *c.local.ticks_since_key_check = 0;
            let mut keys: Vec<Keyboard> = Vec::new();
            if menu_mode {
                keys.push(Keyboard::NoEventIndicated);
            } else {
                (c.shared.buttons, c.shared.encoders, c.shared.config).lock(
                    |buttons, encoders, config| {
                        if buttons.pad0.pressed {
                            keys.push(config.button0.key.to_keyboard());
                        }
                        if buttons.pad1.pressed {
                            keys.push(config.button1.key.to_keyboard());
                        }
                        if buttons.pad2.pressed {
                            keys.push(config.button2.key.to_keyboard());
                        }
                        if buttons.pad3.pressed {
                            keys.push(config.button3.key.to_keyboard());
                        }
                        if buttons.pad4.pressed {
                            keys.push(config.button4.key.to_keyboard());
                        }
                        if buttons.pad5.pressed {
                            keys.push(config.button5.key.to_keyboard());
                        }
                        if buttons.pad6.pressed {
                            keys.push(config.button6.key.to_keyboard());
                        }
                        if buttons.pad7.pressed {
                            keys.push(config.button7.key.to_keyboard());
                        }
                        if buttons.pad8.pressed {
                            keys.push(config.button8.key.to_keyboard());
                        }
                        if buttons.pad9.pressed {
                            keys.push(config.button9.key.to_keyboard());
                        }
                        // Encoder 0
                        let delta = encoders.menu_encoder.read_delta();
                        if delta < 0 {
                            keys.push(config.menu_encoder.left.to_keyboard());
                        } else if delta > 0 {
                            keys.push(config.menu_encoder.right.to_keyboard());
                        }
                        // Encoder 1 push button is reserved for menu navigation,
                        // so we don't check it here
                        // Encoder 2
                        let delta = encoders.encoder1.read_delta();
                        if delta < 0 {
                            keys.push(config.encoder1.left.to_keyboard());
                        } else if delta > 0 {
                            keys.push(config.encoder1.right.to_keyboard());
                        }
                        if encoders.encoder1.button {
                            keys.push(config.encoder1.push.to_keyboard());
                        }
                        // Encoder 3
                        let delta = encoders.encoder2.read_delta();
                        if delta < 0 {
                            keys.push(config.encoder2.left.to_keyboard());
                        } else if delta > 0 {
                            keys.push(config.encoder2.right.to_keyboard());
                        }
                        if encoders.encoder2.button {
                            keys.push(config.encoder2.push.to_keyboard());
                        }
                    },
                );
            }
            // Send keyboard report
            (c.shared.keyboard).lock(|keyboard| match keyboard.device().write_report(keys) {
                Err(UsbHidError::WouldBlock) => {}
                Err(UsbHidError::Duplicate) => {}
                Ok(_) => {}
                Err(e) => {
                    core::panic!("Failed to write keyboard report: {:?}", e)
                }
            });
        }
        *c.local.ticks_since_key_check += 1;
        c.local.keyboard_tick_alarm.clear_interrupt();
        c.local
            .keyboard_tick_alarm
            .schedule(KEYBOARD_UPDATE)
            .unwrap();
    }

    #[task(
        binds = IO_IRQ_BANK0,
        priority = 1,
        shared = [led, encoders, timer, buttons],
        local = [menu_encoder, menu_encoder_switch, encoder1, encoder1_switch, encoder2, encoder2_switch, button0, button1, button2, button3, button4, button5, button6, button7, button8, button9],
    )]
    fn encoder_update(mut c: encoder_update::Context) {
        // Check encoders
        // - menu_encoder
        let menu_encoder_increment = if let Ok(direction) = c.local.menu_encoder.update() {
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
        let menu_encoder_switch_value = c.local.menu_encoder_switch.is_low().unwrap();

        // - encoder1
        let encoder1_increment = if let Ok(direction) = c.local.encoder1.update() {
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
        let encoder1_switch_value = c.local.encoder1_switch.is_low().unwrap();
        // - encoder2
        let encoder2_increment = if let Ok(direction) = c.local.encoder2.update() {
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
        let encoder2_switch_value = c.local.encoder2_switch.is_low().unwrap();

        // Write values

        (c.shared.encoders, c.shared.buttons, c.shared.timer).lock(|encoders, buttons, timer| {
            // - menu_encoder
            let menu_encoder_value = encoders.menu_encoder.value as isize + menu_encoder_increment;
            encoders.menu_encoder.value = if menu_encoder_value < 0 {
                0
            } else {
                menu_encoder_value.try_into().unwrap()
            };
            encoders.menu_encoder.value_changed =
                encoders.menu_encoder.button != menu_encoder_switch_value;
            encoders.menu_encoder.delta += menu_encoder_increment;
            encoders.menu_encoder.button = menu_encoder_switch_value;
            // - encoder1
            let encoder1_value = encoders.encoder1.value as isize + encoder1_increment;
            encoders.encoder1.value = if encoder1_value < 0 {
                0
            } else {
                encoder1_value.try_into().unwrap()
            };
            encoders.encoder1.value_changed = encoders.encoder1.button != encoder1_switch_value;
            encoders.encoder1.delta += encoder1_increment;
            encoders.encoder1.button = encoder1_switch_value;
            // - encoder2
            let encoder2_value = encoders.encoder2.value as isize + encoder2_increment;
            encoders.encoder2.value = if encoder2_value < 0 {
                0
            } else {
                encoder2_value.try_into().unwrap()
            };
            encoders.encoder2.value_changed = encoders.encoder2.button != encoder2_switch_value;
            encoders.encoder2.delta += encoder2_increment;
            encoders.encoder2.button = encoder2_switch_value;
            // - Buttons
            buttons.pad0.update(c.local.button0.is_low().unwrap());
            buttons.pad1.update(c.local.button1.is_low().unwrap());
            buttons.pad2.update(c.local.button2.is_low().unwrap());
            buttons.pad3.update(c.local.button3.is_low().unwrap());
            buttons.pad4.update(c.local.button4.is_low().unwrap());
            buttons.pad5.update(c.local.button5.is_low().unwrap());
            buttons.pad6.update(c.local.button6.is_low().unwrap());
            buttons.pad7.update(c.local.button7.is_low().unwrap());
            buttons.pad8.update(c.local.button8.is_low().unwrap());
            buttons.pad9.update(c.local.button9.is_low().unwrap());
            // Debounce push buttons
            if encoders.menu_encoder.value_changed
                || encoders.encoder1.value_changed
                || encoders.encoder2.value_changed
                || buttons.any_button_changed()
            {
                timer.delay_ms(25u32);
            }

            // Debounce rotary encoders
            if encoders.menu_encoder.delta != 0
                || encoders.encoder1.delta != 0
                || encoders.encoder2.delta != 0
            {
                timer.delay_us(100u32);
            }
        });
    }

    #[task(
        binds = USBCTRL_IRQ,
        priority = 3,
        shared = [keyboard],
        local = [usb_device]
    )]
    fn usb_irq(mut c: usb_irq::Context) {
        c.shared.keyboard.lock(|keyboard| {
            if c.local.usb_device.poll(&mut [keyboard]) {
                let interface = keyboard.device();
                match interface.read_report() {
                    Err(UsbError::WouldBlock) => {}
                    Err(e) => {
                        core::panic!("Failed to read keyboard report: {:?}", e)
                    }
                    Ok(leds) => {}
                }
            }
        })
    }
}
