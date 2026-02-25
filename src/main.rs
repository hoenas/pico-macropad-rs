#![no_std]
#![no_main]

use panic_halt as _;

#[rtic::app(device = rp_pico::hal::pac, peripherals = true)]
mod app {
    use embedded_hal::digital::OutputPin;
    use embedded_sdmmc::{Mode, SdCard, VolumeIdx, VolumeManager};
    use fugit::{MicrosDurationU32, RateExtU32};
    use rotary_encoder_hal::Rotary;
    use rp_pico::hal::{prelude::*, Timer};
    use rp_pico::{
        hal::{
            self, clocks::init_clocks_and_plls, gpio, pac, spi, timer::Alarm, watchdog::Watchdog,
            Sio,
        },
        XOSC_CRYSTAL_FREQ,
    };
    use sh1106::{mode::GraphicsMode, Builder};

    const DISPLAY_UPDATE: MicrosDurationU32 = MicrosDurationU32::millis(50);
    const ROTARY_ENCODER_UPDATE: MicrosDurationU32 = MicrosDurationU32::millis(1);

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
    fn init(c: init::Context) -> (Shared, Local, init::Monotonics) {
        // Soft-reset does not release the hardware spinlocks
        // Release them now to avoid a deadlock after debug or watchdog reset
        unsafe {
            hal::sio::spinlock_reset();
        }
        let mut pac = pac::Peripherals::take().unwrap();
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
        led.set_low().unwrap();
        // Buttons
        let mut button0 = pins.gpio0;
        let mut button1 = pins.gpio1;
        let mut button2 = pins.gpio2;
        let mut button3 = pins.gpio3;
        let mut button4 = pins.gpio4;
        let mut button5 = pins.gpio5;
        let mut button6 = pins.gpio6;
        let mut button7 = pins.gpio7;
        let mut button8 = pins.gpio8;
        let mut button9 = pins.gpio9;
        // Rotary encoders
        let mut rotary_encoder1 = Rotary::new(
            &mut pins.gpio10.into_pull_up_input(),
            &mut pins.gpio11.into_pull_up_input(),
        );
        let mut rotary_encoder_1_button = pins.gpio12;
        let mut rotary_encoder2 = Rotary::new(
            &mut pins.gpio13.into_pull_up_input(),
            &mut pins.gpio14.into_pull_up_input(),
        );
        let mut rotary_encoder_2_button = pins.gpio28;
        // GPIO15 is "DO NOT USE"
        // Display
        let display_sda_pin: hal::gpio::Pin<_, hal::gpio::FunctionI2C, _> =
            pins.gpio26.reconfigure();
        let display_scl_pin: hal::gpio::Pin<_, hal::gpio::FunctionI2C, _> =
            pins.gpio27.reconfigure();
        let display_i2c = hal::I2C::i2c1(
            pac.I2C1,
            display_sda_pin,
            display_scl_pin,
            400.kHz(),
            &mut pac.RESETS,
            &clocks.peripheral_clock,
        );
        let mut display: GraphicsMode<_> = Builder::new().connect_i2c(display_i2c).into();

        // SDCard
        // - Set up our SPI pins into the correct mode
        let spi_sclk: gpio::Pin<_, gpio::FunctionSpi, gpio::PullNone> = pins.gpio18.reconfigure();
        let spi_mosi: gpio::Pin<_, gpio::FunctionSpi, gpio::PullNone> = pins.gpio19.reconfigure();
        let spi_miso: gpio::Pin<_, gpio::FunctionSpi, gpio::PullUp> = pins.gpio20.reconfigure();
        let spi_cs = pins.gpio21.into_push_pull_output();

        // - Create the SPI driver instance for the SPI0 device
        let spi = spi::Spi::<_, _, _, 8>::new(pac.SPI0, (spi_mosi, spi_miso, spi_sclk));

        // - Exchange the uninitialised SPI driver for an initialised one
        let sdmmc_spi = spi.init(
            &mut pac.RESETS,
            clocks.peripheral_clock.freq(),
            400.kHz(), // card initialization happens at low baud rate
            embedded_hal::spi::MODE_0,
        );
        let mut delay = Timer::new(pac.TIMER, &mut pac.RESETS, &clocks);
        let sdcard = SdCard::new(sdmmc_spi, delay);

        // Timers
        // - Display update
        let mut timer = hal::Timer::new(c.device.TIMER, &mut resets, &clocks);
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
