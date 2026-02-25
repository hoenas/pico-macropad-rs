#![no_std]
#![no_main]

use panic_halt as _;

#[rtic::app(device = rp_pico::hal::pac, peripherals = true)]
mod app {

    use embedded_hal::digital::OutputPin;
    use fugit::MicrosDurationU32;
    use rp_pico::{
        hal::{self, clocks::init_clocks_and_plls, timer::Alarm, watchdog::Watchdog, Sio},
        XOSC_CRYSTAL_FREQ,
    };

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
        let mut led = pins.led.reconfigure();
        led.set_low().unwrap();

        // Display update
        let mut timer = hal::Timer::new(c.device.TIMER, &mut resets, &clocks);
        let mut display_alarm = timer.alarm_0().unwrap();
        let _ = display_alarm.schedule(DISPLAY_UPDATE);
        display_alarm.enable_interrupt();

        // Rotary encoder update
        // let mut timer = hal::Timer::new(c.device.TIMER, &mut resets, &clocks);
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
