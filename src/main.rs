#![no_main]
#![no_std]

#[cfg(not(debug_assertions))]
extern crate panic_halt;
#[cfg(debug_assertions)]
extern crate panic_semihosting;

extern crate stm32g0xx_hal as hal;

use core::ptr;
use hal::gpio::{gpioc, Output, PushPull};
use hal::prelude::*;
use hal::rcc;
use hal::stm32::{TIM2, USART2};
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

extern "C" {
    static mut _shared: u32;
}

#[app(device = hal::stm32, peripherals = true)]
const APP: () = {
    struct Resources {
        led: gpioc::PC6<Output<PushPull>>,
        timer: Timer<TIM2>,
        rx: Rx<USART2>,
        tx: Tx<USART2>,
        producer: Producer<'static, u8, U4, u8>,
        consumer: Consumer<'static, u8, U4, u8>,
    }

    #[init]
    fn init(ctx: init::Context) -> init::LateResources {
        log!("init\n");

        let mut rcc = ctx.device.RCC.freeze(rcc::Config::pll()); // 64 MHz
        let gpioa = ctx.device.GPIOA.split(&mut rcc);
        let gpioc = ctx.device.GPIOC.split(&mut rcc);

        let led = gpioc.pc6.into_push_pull_output();

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

        let period = unsafe { ptr::read(&_shared) };
        let period = match period {
            1 => 4,
            4 => 1,
            _ => 1,
        };
        unsafe { ptr::write(&mut _shared, period) };

        let mut timer = ctx.device.TIM2.timer(&mut rcc);
        timer.start(period.hz());
        timer.listen();

        init::LateResources {
            led: led,
            timer: timer,
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

    #[task(binds = USART2, resources = [rx, producer])]
    fn rx(ctx: rx::Context) {
        if let Ok(byte) = block!(ctx.resources.rx.read()) {
            ctx.resources.producer.enqueue(byte).ok();
        }
    }
};
