#![no_std]
#![no_main]

extern crate panic_halt;

use hifive1::hal::delay::Delay;
use hifive1::hal::prelude::*;
use hifive1::hal::spi::{Spi, MODE_0};
use hifive1::hal::DeviceResources;
use hifive1::{pin, sprintln};
use riscv_rt::entry;

mod esp;

#[entry]
fn main() -> ! {
    let deivce_resources = DeviceResources::take().unwrap();
    let peripherals = deivce_resources.peripherals;
    let pins = deivce_resources.pins;
    let clocks = hifive1::clock::configure(peripherals.PRCI, peripherals.AONCLK, 320.mhz().into());
    let mut delay = Delay::new();

    hifive1::stdout::configure(
        peripherals.UART0,
        pin!(pins, uart0_tx),
        pin!(pins, uart0_rx),
        115_200.bps(),
        clocks,
    );

    // TODO: Configure flash speedup, once it isn't buggy

    let mosi = pin!(pins, spi0_mosi).into_iof0();
    let miso = pin!(pins, spi0_miso).into_iof0();
    let sck = pin!(pins, spi0_sck).into_iof0();
    let cs = pin!(pins, spi0_ss2).into_iof0();
    let spi_pins = (mosi, miso, sck, cs);
    let spi = Spi::new(peripherals.QSPI1, spi_pins, MODE_0, 79_207.hz(), clocks);
    let handshake = pins.pin10.into_floating_input();

    let mut wifi = esp::EspWiFi::new(spi, handshake);

    let mut buffer = [0u8; 256];
    wifi.send("AT+CWMODE=0\r\n");
    wifi.recv_blocking(&mut buffer).unwrap();

    loop {
        delay.delay_ms(2000u32);
    }
}
