#![no_main]
#![no_std]

extern crate panic_halt;
extern crate stm32g0xx_hal as hal;

use hal::exti;
use hal::gpio::{gpioa, Output, PushPull, SignalEdge};
use hal::prelude::*;
use hal::rcc;
use hal::stm32::{EXTI, TIM2, USART1};
use hal::time::Hertz;
use hal::timer::Timer;
use hal::{serial, serial::Rx};
use rtfm::app;

macro_rules! log {
    ($msg:expr) => {{
        #[cfg(debug_assertions)]
        cortex_m_semihosting::hio::hstdout()
            .unwrap()
            .write_all($msg.as_bytes())
            .unwrap();
    }};
}

#[app(device = hal::stm32, peripherals = true)]
const APP: () = {
    struct Resources {
        led: gpioa::PA12<Output<PushPull>>,
        timer: Timer<TIM2>,
        period: Hertz,
        exti: EXTI,
        rx: Rx<USART1>,
    }

    #[init]
    fn init(ctx: init::Context) -> init::LateResources {
        log!("init\n");

        let mut rcc = ctx.device.RCC.freeze(rcc::Config::pll()); // 64 MHz
        let mut exti = ctx.device.EXTI;
        let gpioa = ctx.device.GPIOA.split(&mut rcc);
        let gpiob = ctx.device.GPIOB.split(&mut rcc);

        let led = gpioa.pa12.into_push_pull_output();
        let _btn = gpioa
            .pa0
            .listen(SignalEdge::Falling, &mut exti)
            .into_pull_up_input();

        let (_tx, mut rx) = ctx
            .device
            .USART1
            .usart(
                gpioa.pa9,
                gpiob.pb7,
                serial::Config::default().baudrate(115200.bps()),
                &mut rcc,
            )
            .unwrap()
            .split();

        rx.listen();

        let period = 1.hz();
        let mut timer = ctx.device.TIM2.timer(&mut rcc);
        timer.start(period);
        timer.listen();

        init::LateResources {
            led,
            timer,
            period,
            exti,
            rx,
        }
    }

    #[idle]
    fn idle(_: idle::Context) -> ! {
        loop {
            log!("idle\n");
            rtfm::export::wfi();
        }
    }

    #[task(binds = TIM2, resources = [led, timer])]
    fn blink(ctx: blink::Context) {
        log!("blink\n");
        ctx.resources.led.toggle().unwrap();
        ctx.resources.timer.clear_irq();
    }

    #[task(binds = EXTI0_1, resources = [exti, timer, period])]
    fn button(ctx: button::Context) {
        log!("button\n");
        let period = ctx.resources.period;
        if *period == 1.hz() {
            *period = 4.hz();
            ctx.resources.timer.start(*period);
        } else {
            *period = 1.hz();
            ctx.resources.timer.start(*period);
        }
        ctx.resources.exti.unpend(exti::Event::GPIO0);
    }

    #[task(binds = USART1, resources = [rx])]
    fn rx(ctx: rx::Context) {
        if let Ok(byte) = ctx.resources.rx.read() {}
    }
};
