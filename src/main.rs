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

#[derive(Eq, PartialEq)]
enum State {
    Good,
    Bad,
}

const MAX_BAD_EVENTS: u32 = 3;

#[entry]
fn main() -> ! {
    let deivce_resources = DeviceResources::take().unwrap();
    let peripherals = deivce_resources.peripherals;
    let pins = deivce_resources.pins;
    // Run slower than the regular 320MHz to save power
    let clocks = hifive1::clock::configure(peripherals.PRCI, peripherals.AONCLK, 50.mhz().into());
    let mut delay = Delay::new();

    hifive1::stdout::configure(
        peripherals.UART0,
        pin!(pins, uart0_tx),
        pin!(pins, uart0_rx),
        115_200.bps(),
        clocks,
    );

    let mosi = pin!(pins, spi0_mosi).into_iof0();
    let miso = pin!(pins, spi0_miso).into_iof0();
    let sck = pin!(pins, spi0_sck).into_iof0();
    let cs = pin!(pins, spi0_ss2).into_iof0();
    let spi_pins = (mosi, miso, sck, cs);
    let spi = Spi::new(peripherals.QSPI1, spi_pins, MODE_0, 79_207.hz(), clocks);
    let handshake = pins.pin10.into_floating_input();
    let mut red_led = pin!(pins, led_red).into_output();
    let mut green_led = pin!(pins, led_green).into_output();
    let mut relay_pin = pin!(pins, dig19).into_output();

    relay_pin.set_low().unwrap();
    red_led.set_high().unwrap();
    green_led.set_high().unwrap();

    let mut wifi = esp::EspWiFi::new(spi, handshake);
    let mut bad_event_counter: u32 = 0;
    // Bootup likely happens after a power outage, which means it's good to assume that things are
    // probably bad.
    let mut state = State::Bad;

    loop {
        let internet_status = wifi.is_internet_ok();

        if internet_status.is_ok() {
            red_led.set_high().unwrap();
            green_led.set_low().unwrap();
            state = State::Good;
            bad_event_counter = 0;
        } else {
            if state == State::Good {
                if bad_event_counter == MAX_BAD_EVENTS {
                    state = State::Bad;
                    bad_event_counter = 0;
                } else {
                    bad_event_counter += 1;
                }
            }
            red_led.set_low().unwrap();
            green_led.set_high().unwrap();
        }

        if state == State::Bad {
            relay_pin.set_high().unwrap();
        } else {
            relay_pin.set_low().unwrap();
        }

        sprintln!("Internet status: {:?}", internet_status);

        // If we are in a transitionary state, poll more quickly
        if bad_event_counter == 0 && state == State::Good {
            delay.delay_ms(300_000u32);
        } else {
            delay.delay_ms(60_000u32);
        }
    }
}
