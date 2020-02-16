#![no_main]
#![no_std]

#[cfg(not(debug_assertions))]
extern crate panic_halt;
#[cfg(debug_assertions)]
extern crate panic_semihosting;

extern crate stm32g0xx_hal as hal;

use hal::exti;
use hal::gpio::{gpioc, Output, PushPull, SignalEdge};
use hal::prelude::*;
use hal::rcc;
use hal::rcc::ResetMode;
use hal::stm32::{EXTI, TIM2, USART2};
use hal::time::Hertz;
use hal::timer::Timer;
use hal::{serial, serial::Rx, serial::Tx};
use heapless::{
    consts::U4,
    i::Queue as ConstQueue,
    spsc::{Consumer, Producer, Queue},
};
use nb::block;
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
        led: gpioc::PC6<Output<PushPull>>,
        timer: Timer<TIM2>,
        period: Hertz,
        //exti: EXTI,
        rx: Rx<USART2>,
        tx: Tx<USART2>,
        producer: Producer<'static, u8, U4, u8>,
        consumer: Consumer<'static, u8, U4, u8>,
    }

    #[init]
    fn init(ctx: init::Context) -> init::LateResources {
        log!("init\n");

        let mut rcc = ctx.device.RCC.freeze(rcc::Config::pll()); // 64 MHz
                                                                 //let mut exti = ctx.device.EXTI;
        let gpioa = ctx.device.GPIOA.split(&mut rcc);
        let gpioc = ctx.device.GPIOC.split(&mut rcc);
        //let gpiof = ctx.device.GPIOF.split(&mut rcc);

        let led = gpioc.pc6.into_push_pull_output();

        // let _btn = gpiof
        //     .pf2
        //     .listen(SignalEdge::Falling, &mut exti)
        //     .into_pull_up_input();

        static mut Q: Queue<u8, U4, u8> = Queue(ConstQueue::u8());
        let (producer, consumer) = unsafe { Q.split() };

        let (tx, mut rx) = ctx
            .device
            .USART2
            .usart(
                gpioa.pa2,
                gpioa.pa3,
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
            led: led,
            timer: timer,
            period: period,
            //exti: exti,
            rx: rx,
            tx: tx,
            producer: producer,
            consumer: consumer,
        }
    }

    #[idle(resources = [tx, consumer])]
    fn idle(ctx: idle::Context) -> ! {
        loop {
            log!("idle\n");
            while let Some(byte) = ctx.resources.consumer.dequeue() {
                let byte = (byte as char).to_ascii_uppercase() as u8;
                block!(ctx.resources.tx.write(byte)).ok();
            }
            rtfm::export::wfi();
        }
    }

    #[task(binds = TIM2, resources = [led, timer])]
    fn blink(ctx: blink::Context) {
        log!("blink\n");
        ctx.resources.led.toggle().ok();
        ctx.resources.timer.clear_irq();
    }

    // #[task(binds = EXTI2_3, resources = [exti, timer, period])]
    // fn button(ctx: button::Context) {
    //     log!("button\n");
    //     let period = ctx.resources.period;
    //     if *period == 1.hz() {
    //         *period = 4.hz();
    //         ctx.resources.timer.start(*period);
    //     } else {
    //         *period = 1.hz();
    //         ctx.resources.timer.start(*period);
    //     }
    //     ctx.resources.exti.unpend(exti::Event::GPIO2);
    //     //let _rpr1 = &ctx.resources.exti.rpr1;
    //     let _fpr1 = &ctx.resources.exti.fpr1;
    //     let _x = 5;
    // }

    #[task(binds = USART2, resources = [rx, producer])]
    fn rx(ctx: rx::Context) {
        if let Ok(byte) = block!(ctx.resources.rx.read()) {
            ctx.resources.producer.enqueue(byte).ok();
        }
    }
};
