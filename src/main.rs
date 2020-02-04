#![no_main]
#![no_std]

extern crate panic_halt;
extern crate stm32g0xx_hal as hal;

use hal::exti::Event;
use hal::gpio::{gpioa, Output, PushPull, SignalEdge};
use hal::prelude::*;
use hal::stm32::{EXTI, TIM3};
use hal::timer::Timer;
use rtfm::app;

mod debug; // semihosting print

#[app(device = hal::stm32, peripherals = true)]
const APP: () = {
    struct Resources {
        led: gpioa::PA12<Output<PushPull>>,
        timer: Timer<TIM3>,
        exti: EXTI,
    }

    #[init]
    fn init(ctx: init::Context) -> init::LateResources {
        debug!("init");

        let mut rcc = ctx.device.RCC.constrain();
        let mut exti = ctx.device.EXTI;
        let gpioa = ctx.device.GPIOA.split(&mut rcc);
        let led = gpioa.pa12.into_push_pull_output();
        let _btn = gpioa
            .pa0
            .listen(SignalEdge::Falling, &mut exti)
            .into_pull_up_input();

        let mut timer = ctx.device.TIM3.timer(&mut rcc);
        timer.start(1.hz());
        timer.listen();

        init::LateResources { led, timer, exti }
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

    #[task(binds = TIM3, resources = [led, timer])]
    fn blink(ctx: blink::Context) {
        static mut COUNT: u32 = 0; // this variable must be static
        debug!("blink {}", COUNT);
        *COUNT += 1;
        ctx.resources.led.toggle().unwrap();
        ctx.resources.timer.clear_irq();
    }

    #[task(binds = EXTI0_1, resources = [exti])]
    fn button(ctx: button::Context) {
        static mut COUNT: u32 = 0; // this variable must be static
        debug!("button {}", COUNT);
        *COUNT += 1;
        ctx.resources.exti.unpend(Event::GPIO0);
    }
};
