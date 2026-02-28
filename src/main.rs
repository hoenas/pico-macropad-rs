#![no_std]
#![no_main]

use panic_halt as _;

#[rtic::app(device = rp_pico::hal::pac, peripherals = true)]
mod app {
    use embedded_hal::digital::StatefulOutputPin;
    use embedded_sdmmc::sdcard;
    use fugit::MicrosDurationU32;
    use rp_pico::XOSC_CRYSTAL_FREQ;
    // The macro for our start-up function
    use rp_pico::entry;

    // info!() and error!() macros for printing information to the debug output
    use defmt::*;
    use defmt_rtt as _;

    // Ensure we halt the program on panic (if we don't mention this crate it won't
    // be linked)
    use panic_halt as _;

    use rp_pico::hal::clocks::init_clocks_and_plls;
    use rp_pico::hal::Sio;
    use rp_pico::hal::Watchdog;
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
    use embedded_sdmmc::{SdCard, TimeSource, Timestamp, VolumeIdx, VolumeManager};

    // Get the file open mode enum:
    use embedded_sdmmc::filesystem::Mode;

    use embedded_hal::delay::DelayNs;
    use embedded_hal::digital::OutputPin;
    use rp_pico::hal::Timer;

    use rotary_encoder_hal::Rotary;
    use rp_pico::hal::timer::Alarm;
    use sh1106::{mode::GraphicsMode, Builder};
    use ws2812_pio::Ws2812;

    const DISPLAY_UPDATE: MicrosDurationU32 = MicrosDurationU32::millis(50);
    const ROTARY_ENCODER_UPDATE: MicrosDurationU32 = MicrosDurationU32::millis(1);

    /// A dummy timesource, which is mostly important for creating files.
    #[derive(Default)]
    pub struct DummyTimesource();

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
    struct Shared {
        timer: hal::Timer,
        display_alarm: hal::timer::Alarm0,
        rotary_encoder_alarm: hal::timer::Alarm1,
        led: hal::gpio::Pin<
            hal::gpio::bank0::Gpio25,
            hal::gpio::FunctionSioOutput,
            hal::gpio::PullNone,
        >,
    }

    #[local]
    struct Local {}

    #[init]
    fn init(mut c: init::Context) -> (Shared, Local, init::Monotonics) {
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
        let mut button0 = pins.gpio0.into_pull_up_input();
        let mut button1 = pins.gpio1.into_pull_up_input();
        let mut button2 = pins.gpio2.into_pull_up_input();
        let mut button3 = pins.gpio3.into_pull_up_input();
        let mut button4 = pins.gpio4.into_pull_up_input();
        let mut button5 = pins.gpio5.into_pull_up_input();
        let mut button6 = pins.gpio6.into_pull_up_input();
        let mut button7 = pins.gpio7.into_pull_up_input();
        let mut button8 = pins.gpio8.into_pull_up_input();
        let mut button9 = pins.gpio9.into_pull_up_input();
        // Rotary encoders
        // - Encoder 1
        let mut rotary_encoder1 = Rotary::new(
            &mut pins.gpio10.into_pull_up_input(),
            &mut pins.gpio11.into_pull_up_input(),
        );
        let mut rotary_encoder_1_button = pins.gpio12;
        // - Encoder 2
        let mut rotary_encoder2 = Rotary::new(
            &mut pins.gpio13.into_pull_up_input(),
            &mut pins.gpio14.into_pull_up_input(),
        );
        let mut rotary_encoder_2_button = pins.gpio15;
        // - Encoder 3
        let mut rotary_encoder3 = Rotary::new(
            &mut pins.gpio20.into_pull_up_input(),
            &mut pins.gpio21.into_pull_up_input(),
        );
        let mut rotary_encoder_3_button = pins.gpio22;
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
        let mut spi0 = spi::Spi::<_, _, _, 8>::new(
            c.device.SPI0,
            (sdmmc_spi_mosi, sdmmc_spi_miso, sdmmc_spi_sclk),
        );

        // - Exchange the uninitialised SPI driver for an initialised one
        let mut sdmmc_spi = spi0.init(
            &mut resets,
            clocks.peripheral_clock.freq(),
            400.kHz(), // card initialization happens at low baud rate
            embedded_hal::spi::MODE_0,
        );
        let mut timer = Timer::new(c.device.TIMER, &mut resets, &clocks);
        let sdcard = SdCard::new(sdmmc_spi, sdmmc_spi_cs, timer);
        let mut volume_mgr = VolumeManager::new(sdcard, DummyTimesource::default());

        // - RGB LED
        let (mut pio, sm0, _, _, _) = c.device.PIO0.split(&mut resets);
        let mut rgb_led = Ws2812::new(
            pins.gpio28.into_function(),
            &mut pio,
            sm0,
            clocks.peripheral_clock.freq(),
            timer.count_down(),
        );

        // Timers
        // - Display update
        let mut display_alarm = timer.alarm_0().unwrap();
        let _ = display_alarm.schedule(DISPLAY_UPDATE);
        display_alarm.enable_interrupt();
        // - Rotary encoder update
        let mut rotary_encoder_alarm = timer.alarm_1().unwrap();
        let _ = rotary_encoder_alarm.schedule(ROTARY_ENCODER_UPDATE);
        rotary_encoder_alarm.enable_interrupt();

        (
            Shared {
                timer,
                display_alarm,
                rotary_encoder_alarm,
                led,
            },
            Local {},
            init::Monotonics(),
        )
    }

    #[task(
        binds = TIMER_IRQ_0,
        priority = 1,
        shared = [timer, display_alarm, led],
        local = [tog: bool = true],
    )]
    fn display_update(mut c: display_update::Context) {
        c.shared.led.lock(|l| l.set_high().unwrap());

        let mut alarm = c.shared.display_alarm;
        (alarm).lock(|a| {
            a.clear_interrupt();
            let _ = a.schedule(DISPLAY_UPDATE);
        });
    }

    #[task(
        binds = TIMER_IRQ_1,
        priority = 1,
        shared = [timer, rotary_encoder_alarm, led],
        local = [tog: bool = true],
    )]
    fn rotary_encoder_update(mut c: rotary_encoder_update::Context) {
        c.shared.led.lock(|l| l.set_low().unwrap());

        *c.local.tog = !*c.local.tog;

        let mut alarm = c.shared.rotary_encoder_alarm;
        (alarm).lock(|a| {
            a.clear_interrupt();
            let _ = a.schedule(DISPLAY_UPDATE);
        });
    }
}
