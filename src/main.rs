#![no_std]
#![no_main]

extern crate cortex_m;
extern crate cortex_m_rt as rt;
extern crate panic_halt;
extern crate stm32g0xx_hal as hal;

use hal::prelude::*;
use hal::stm32;
use rt::entry;

mod semihosting; // semihosting println!() macro

#[entry]
fn main() -> ! {
    println!("Hello, World!");
    let dp = stm32::Peripherals::take().expect("cannot take peripherals");
    let mut rcc = dp.RCC.constrain();
    let gpioa = dp.GPIOA.split(&mut rcc);
    let mut led = gpioa.pa12.into_push_pull_output();

    loop {
        println!("led.set_low()");
        for _ in 0..10_000 {
            led.set_low().unwrap();
        }
        println!("led.set_high()");
        for _ in 0..10_000 {
            led.set_high().unwrap();
        }
    }
}
