#![no_main]
#![no_std]

use bsp::{hal, pin_alias};
use hal::fugit::MillisDurationU32;
use hal::{
    async_hal::timer::TimerFuture, clock::GenericClockController, ehal::digital::StatefulOutputPin,
    pac::Tc4, timer::TimerCounter,
};
use pygamer as bsp;

atsamd_hal::bind_interrupts!(struct Irqs {
    TC4 => atsamd_hal::async_hal::timer::InterruptHandler<Tc4>;
});

#[rtic::app(device = bsp::pac, dispatchers = [EVSYS_0])]
mod app {
    use super::*;

    #[shared]
    struct Shared {}

    #[local]
    struct Local {}

    #[init]
    fn init(cx: init::Context) -> (Shared, Local) {
        let mut peripherals = cx.device;
        let _core = cx.core;

        let mut clocks = GenericClockController::with_external_32kosc(
            peripherals.gclk,
            &mut peripherals.mclk,
            &mut peripherals.osc32kctrl,
            &mut peripherals.oscctrl,
            &mut peripherals.nvmctrl,
        );
        let pins = bsp::Pins::new(peripherals.port);
        let red_led: bsp::RedLed = pin_alias!(pins.red_led).into();

        // configure a clock for the TC4 and TC5 peripherals
        let timer_clock = clocks.gclk0();
        let tc45 = &clocks.tc4_tc5(&timer_clock).unwrap();

        // instantiate a timer object for the TC4 peripheral
        let timer = TimerCounter::tc4_(tc45, peripherals.tc4, &mut peripherals.mclk);
        let timer = timer.into_future(Irqs);

        let _ = timer::spawn(timer, red_led);

        (
            Shared {},
            // initial values for the `#[local]` resources
            Local {},
        )
    }

    #[idle]
    fn idle(_cx: idle::Context) -> ! {
        loop {
            cortex_m::asm::wfi();
        }
    }

    #[task(priority = 1)]
    async fn timer(_cx: timer::Context, mut timer: TimerFuture<Tc4>, mut red_led: bsp::RedLed) {
        loop {
            timer
                .delay(MillisDurationU32::from_ticks(500).convert())
                .await;
            red_led.toggle().unwrap();
        }
    }
}
