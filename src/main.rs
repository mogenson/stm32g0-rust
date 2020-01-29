#![no_std]
#![no_main]

extern crate panic_halt;

use cortex_m::asm;
use cortex_m_rt::entry;
use cortex_m_semihosting::hprintln;

#[entry]
fn main() -> ! {
    asm::nop(); // To not have main optimize to abort in release mode, remove when you add code

    hprintln!("Hello, World!").unwrap();

    loop {
        // your code goes here
    }
}
