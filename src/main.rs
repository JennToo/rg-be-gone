#![no_std]
#![no_main]

extern crate panic_halt;

use hifive1::hal::delay::Delay;
use hifive1::hal::prelude::*;
use hifive1::hal::DeviceResources;
use hifive1::pin;
use riscv_rt::entry;


#[entry]
fn main() -> ! {
    let deivce_resources = DeviceResources::take().unwrap();
    let peripherals = deivce_resources.peripherals;
    let pins = deivce_resources.pins;
    let clocks = hifive1::clock::configure(peripherals.PRCI, peripherals.AONCLK, 320.mhz().into());
    let mut delay = Delay::new();

    // TODO: Configure flash speedup, once it isn't buggy

    loop {
        // TODO: Check
    }
}
