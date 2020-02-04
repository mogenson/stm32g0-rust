#![no_main]
#![no_std]

extern crate panic_halt;
extern crate stm32g0xx_hal as hal;

use hal::gpio::{gpioa::PA12, Output, PushPull};
use hal::prelude::*;
use hal::stm32::TIM2;
use hal::timer::Timer;
use rtfm::app;

mod debug; // semihosting print

#[app(device = hal::stm32, peripherals = true)]
const APP: () = {
    struct Resources {
        led: PA12<Output<PushPull>>,
        timer: Timer<TIM2>,
    }

    #[init]
    fn init(ctx: init::Context) -> init::LateResources {
        debug!("init");

        let mut rcc = ctx.device.RCC.constrain();
        let gpioa = ctx.device.GPIOA.split(&mut rcc);
        let led = gpioa.pa12.into_push_pull_output();

        let mut timer = ctx.device.TIM2.timer(&mut rcc);
        timer.start(1.hz());
        timer.listen();

        init::LateResources { led, timer }
    }

    #[idle]
    fn idle(_: idle::Context) -> ! {
        let mut count: u32 = 0; // this variable can be local
        loop {
            debug!("idle {}", count);
            count += 1;
            rtfm::export::wfi();
        }
    }

    #[task(binds = TIM2, resources = [led, timer])]
    fn blink(ctx: blink::Context) {
        static mut COUNT: u32 = 0; // this variable must be static
        debug!("blink {}", COUNT);
        *COUNT += 1;
        ctx.resources.led.toggle().unwrap();
        ctx.resources.timer.clear_irq();
    }
};
