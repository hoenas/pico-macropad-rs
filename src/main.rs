#![no_std]
#![no_main]

use panic_halt as _;

#[rtic::app(device = rp_pico::hal::pac, peripherals = true)]
mod app {

    extern crate alloc;

    use alloc::string::String;
    use alloc::vec::Vec;
    use embedded_alloc::Heap;
    use embedded_hal::digital::StatefulOutputPin;
    use embedded_menu::items::MenuItem;

    use fugit::MicrosDurationU32;
    use rotary_encoder_hal::DefaultPhase;
    use rp_pico::XOSC_CRYSTAL_FREQ;
    // The macro for our start-up function

    // info!() and error!() macros for printing information to the debug output
    use defmt::*;
    use defmt_rtt as _;

    // Ensure we halt the program on panic (if we don't mention this crate it won't
    // be linked)
    use panic_halt as _;

    use hal::gpio::Pin;

    use rp2040_hal::gpio::FunctionPio0;
    use rp2040_hal::gpio::Interrupt::{EdgeHigh, EdgeLow};
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
    use rp_pico::pac::PIO0;
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
    use embedded_sdmmc::{SdCard, TimeSource, Timestamp};

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
    use rp2040_hal::timer::CountDown;
    use rp_pico::hal::timer::Alarm;
    use rp_pico::pac::I2C1;
    use sh1106::{prelude::*, Builder};
    use smart_leds::SmartLedsWrite;
    use smart_leds::RGB8;
    use ws2812_pio::Ws2812;
    // USB Device support
    use usb_device::class_prelude::*;
    // USB Human Interface Device (HID) Class support
    use embedded_hal_bus::spi::ExclusiveDevice;

    const DISPLAY_UPDATE: MicrosDurationU32 = MicrosDurationU32::millis(50);
    const RGB_LEDS_UPDATE: MicrosDurationU32 = MicrosDurationU32::millis(25);
    const NUM_LEDS: usize = 7;
    const MAX_FILE_NAMES: usize = 64;
    const CHARACTER_STYLE: MonoTextStyle<BinaryColor> =
        MonoTextStyle::new(&FONT_6X10, BinaryColor::On);
    const MENU_ENTRY_COUNT: usize = 6;
    /// A dummy timesource, which is mostly important for creating files.
    #[derive(Default)]
    struct DummyTimesource();

    impl TimeSource for DummyTimesource {
        // In theory you could use the RTC of the rp2040 here, if you had
        // any external time synchronizing device.
        fn get_timestamp(&self) -> Timestamp {
            Timestamp {
                year_since_1970: 0,
                zero_indexed_month: 0,
                zero_indexed_day: 0,
                hours: 0,
                minutes: 0,
                seconds: 0,
            }
        }
    }

    #[shared]
    struct Shared<'a> {
        timer: hal::Timer,
        display_alarm: hal::timer::Alarm0,
        rgb_leds_alarm: hal::timer::Alarm1,
        led: Pin<Gpio25, FunctionSioOutput, PullNone>,
        rotary_encoder1: Rotary<
            Pin<Gpio10, FunctionSio<SioInput>, PullNone>,
            Pin<Gpio11, FunctionSio<SioInput>, PullNone>,
            DefaultPhase,
        >,
        rotary_encoder1_value: usize,
        rotary_encoder1_switch: Pin<Gpio12, FunctionSio<SioInput>, PullUp>,
        rotary_encoder2: Rotary<
            Pin<Gpio13, FunctionSio<SioInput>, PullNone>,
            Pin<Gpio14, FunctionSio<SioInput>, PullNone>,
            DefaultPhase,
        >,
        rotary_encoder2_value: usize,
        rotary_encoder2_switch: Pin<Gpio15, FunctionSio<SioInput>, PullUp>,
        rotary_encoder3: Rotary<
            Pin<Gpio20, FunctionSio<SioInput>, PullNone>,
            Pin<Gpio21, FunctionSio<SioInput>, PullNone>,
            DefaultPhase,
        >,
        rotary_encoder3_value: usize,
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
    }

    // Setup some blinking codes:
    const BLINK_OK_LONG: [u8; 1] = [8u8];
    const BLINK_OK_SHORT_LONG: [u8; 4] = [1u8, 0u8, 6u8, 0u8];
    const BLINK_OK_SHORT_SHORT_LONG: [u8; 6] = [1u8, 0u8, 1u8, 0u8, 6u8, 0u8];
    const BLINK_ERR_3_SHORT: [u8; 6] = [1u8, 0u8, 1u8, 0u8, 1u8, 0u8];
    const BLINK_ERR_4_SHORT: [u8; 8] = [1u8, 0u8, 1u8, 0u8, 1u8, 0u8, 1u8, 0u8];
    const BLINK_ERR_5_SHORT: [u8; 10] = [1u8, 0u8, 1u8, 0u8, 1u8, 0u8, 1u8, 0u8, 1u8, 0u8];
    const BLINK_ERR_6_SHORT: [u8; 12] =
        [1u8, 0u8, 1u8, 0u8, 1u8, 0u8, 1u8, 0u8, 1u8, 0u8, 1u8, 0u8];

    fn blink_signals(
        pin: &mut dyn embedded_hal::digital::OutputPin<Error = core::convert::Infallible>,
        delay: &mut dyn DelayNs,
        sig: &[u8],
    ) {
        for bit in sig {
            if *bit != 0 {
                pin.set_high().unwrap();
            } else {
                pin.set_low().unwrap();
            }

            let length = if *bit > 0 { *bit } else { 1 };

            for _ in 0..length {
                delay.delay_ms(100);
            }
        }

        pin.set_low().unwrap();

        delay.delay_ms(500);
    }

    fn blink_signals_loop(
        pin: &mut dyn embedded_hal::digital::OutputPin<Error = core::convert::Infallible>,
        delay: &mut dyn DelayNs,
        sig: &[u8],
    ) -> ! {
        loop {
            blink_signals(pin, delay, sig);
        }
    }

    #[local]
    struct Local {}

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
        let volume_mgr = embedded_sdmmc::VolumeManager::new(sdcard, DummyTimesource::default());
        Text::with_alignment(
            "Opening SDCard...",
            display.bounding_box().top_left + Point::new(0, 10),
            CHARACTER_STYLE,
            Alignment::Left,
        )
        .draw(&mut display)
        .unwrap();
        display.flush().unwrap();
        let volume0 = match volume_mgr.open_volume(embedded_sdmmc::VolumeIdx(0)) {
            Err(_) => blink_signals_loop(&mut led, &mut timer, &BLINK_ERR_3_SHORT),
            Ok(val) => val,
        };
        Text::with_alignment(
            "Reading files...",
            display.bounding_box().top_left + Point::new(0, 20),
            CHARACTER_STYLE,
            Alignment::Left,
        )
        .draw(&mut display)
        .unwrap();
        display.flush().unwrap();

        let root_dir = match volume0.open_root_dir() {
            Err(_) => blink_signals_loop(&mut led, &mut timer, &BLINK_ERR_4_SHORT),
            Ok(val) => val,
        };
        let mut filename_buffer = [0_u8; 50];
        let mut lfn_buffer = embedded_sdmmc::LfnBuffer::new(&mut filename_buffer);
        let mut menu_items = Vec::new();
        root_dir
            .iterate_dir_lfn(&mut lfn_buffer, |_, filename| {
                if let Some(filename) = filename {
                    if filename.ends_with(".json") {
                        menu_items.push(MenuItem::new(String::from(filename), ""));
                    }
                }
            })
            .unwrap();
        let style =
            embedded_menu::MenuStyle::new(BinaryColor::On).with_animated_selection_indicator(10);
        let mut menu = embedded_menu::Menu::with_style("Load Config", style)
            .add_menu_items(menu_items)
            .build();
        display.clear();
        menu.update(&display);
        menu.draw(&mut display).unwrap();
        display.flush().unwrap();
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
                display_alarm,
                rgb_leds_alarm,
                led,
                rotary_encoder1,
                rotary_encoder1_switch,
                rotary_encoder1_value: 0,
                rotary_encoder2,
                rotary_encoder2_switch,
                rotary_encoder2_value: 0,
                rotary_encoder3,
                rotary_encoder3_switch,
                rotary_encoder3_value: 0,
                display,
                rgb_leds,
            },
            Local {},
            init::Monotonics(),
        )
    }

    #[task(
        binds = TIMER_IRQ_0,
        priority = 3,
        shared = [timer, display_alarm, led, display, rotary_encoder1_value, rotary_encoder2_value, rotary_encoder3_value],
        local = [tog: bool = true],
    )]
    fn display_update(mut c: display_update::Context) {
        c.shared.led.lock(|l| l.set_high().unwrap());
        let mut rotary1_value = 0;
        let mut rotary2_value = 0;
        let mut rotary3_value = 0;
        c.shared
            .rotary_encoder1_value
            .lock(|value| rotary1_value = *value);
        c.shared
            .rotary_encoder2_value
            .lock(|value| rotary2_value = *value);
        c.shared
            .rotary_encoder3_value
            .lock(|value| rotary3_value = *value);
        let _menu_index = (rotary1_value / 4);

        let mut alarm = c.shared.display_alarm;
        (alarm).lock(|a| {
            a.clear_interrupt();
            let _ = a.schedule(DISPLAY_UPDATE);
        });
    }

    #[task(
        binds = TIMER_IRQ_1,
        priority = 4,
        shared = [timer, rgb_leds_alarm, rgb_leds, ],
        local = [tog: bool = true, animation_counter: usize = 0],
    )]
    fn leds_update(mut c: leds_update::Context) {
        *c.local.animation_counter += 1;
        let _counter = c.local.animation_counter;

        c.shared.rgb_leds.lock(|rgb_leds| {
            // Write RGB values
            let mut data: [RGB8; NUM_LEDS] = [RGB8::default(); NUM_LEDS];
            for i in 0..data.len() {
                data[i] = RGB8::new(50, 0, 50);
            }
            rgb_leds.write(data.iter().cloned());
        });

        let mut alarm = c.shared.rgb_leds_alarm;
        (alarm).lock(|a| {
            a.clear_interrupt();
            let _ = a.schedule(RGB_LEDS_UPDATE);
        });
    }

    #[task(
        binds = IO_IRQ_BANK0,
        priority = 1,
        shared = [led, rotary_encoder1, rotary_encoder1_switch, rotary_encoder1_value, rotary_encoder2, rotary_encoder2_switch,rotary_encoder2_value, rotary_encoder3, rotary_encoder3_switch, rotary_encoder3_value],
        local = [tog: bool = true],
    )]
    fn rotary_encoder_update(mut c: rotary_encoder_update::Context) {
        c.shared.led.lock(|l| l.set_low().unwrap());

        *c.local.tog = !*c.local.tog;

        // Check encoders
        // - Encoder1
        c.shared.rotary_encoder1.lock(|encoder| {
            c.shared.rotary_encoder1_value.lock(|value| {
                let increment = if let Ok(direction) = encoder.update() {
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
                let temp_value = *value as i32 + increment;
                *value = if temp_value < 0 {
                    0
                } else {
                    temp_value as usize
                };
            });
        });
        c.shared.rotary_encoder1_switch.lock(|_switch| {});

        // - Encoder2
        c.shared.rotary_encoder2.lock(|encoder| {
            c.shared.rotary_encoder2_value.lock(|value| {
                let increment = if let Ok(direction) = encoder.update() {
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
                let temp_value = *value as i32 + increment;
                *value = if temp_value < 0 {
                    0
                } else {
                    temp_value as usize
                };
            });
        });
        // - Encoder1
        c.shared.rotary_encoder3.lock(|encoder| {
            c.shared.rotary_encoder3_value.lock(|value| {
                let increment = if let Ok(direction) = encoder.update() {
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
                let temp_value = *value as i32 + increment;
                *value = if temp_value < 0 {
                    0
                } else {
                    temp_value as usize
                };
            });
        });
    }
}
