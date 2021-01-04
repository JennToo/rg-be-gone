use embedded_hal::blocking::spi::WriteIter;
use hifive1::hal::delay::Delay;
use hifive1::hal::gpio::{gpio0::Pin10, Floating, Input};
use hifive1::hal::prelude::*;
use hifive1::hal::spi::{Spi, SpiX};

#[derive(Debug)]
pub enum EspError {
    ProtocolError,
    BufferOverflow,
    WouldBlock,
}

pub struct EspWiFi<SPI, PINS> {
    spi: Spi<SPI, PINS>,
    handshake: Pin10<Input<Floating>>,
}

impl<SPI: SpiX, PINS> EspWiFi<SPI, PINS> {
    pub fn new(spi: Spi<SPI, PINS>, handshake: Pin10<Input<Floating>>) -> Self {
        Self { spi, handshake }
    }

    fn send_bytes(&mut self, bytes: &[u8]) {
        self.spi.write(bytes).unwrap();
    }

    fn transfer(&mut self, buffer: &mut [u8]) {
        self.spi.transfer(buffer).unwrap();
    }

    fn discard(&mut self, size: usize) {
        self.spi.write_iter((0..size).map(|_| 0x00)).unwrap();
    }

    fn wait_for_ready(&mut self) {
        while self.handshake.is_low().unwrap() {}
    }

    pub fn send(&mut self, s: &str) {
        let bytes = s.as_bytes();
        assert!(bytes.len() <= 127);

        self.send_bytes(&[0x02, 0x00, 0x00, 0x00]);
        Delay.delay_ms(5u32);
        self.send_bytes(&[bytes.len() as u8, 0x00, 0x00, 0x41]);
        Delay.delay_ms(5u32);
        self.send_bytes(bytes);
        Delay.delay_ms(15u32);
    }

    pub fn recv_blocking<'a>(&mut self, buffer: &'a mut [u8]) -> Result<&'a str, EspError> {
        self.wait_for_ready();
        self.recv(buffer)
    }

    pub fn recv<'a>(&mut self, buffer: &'a mut [u8]) -> Result<&'a str, EspError> {
        if self.handshake.is_low().unwrap() {
            return Err(EspError::WouldBlock);
        }

        self.send_bytes(&[0x01, 0x00, 0x00, 0x00]);
        Delay.delay_ms(5u32);

        let mut request = [0u8; 4];
        self.transfer(&mut request);
        Delay.delay_ms(5u32);
        if request[3] != 0x42 {
            return Err(EspError::ProtocolError);
        }

        let n = (request[0] & 0x7F) as usize + ((request[1] as usize) << 7);
        if n > buffer.len() {
            self.discard(n);
            return Err(EspError::BufferOverflow);
        }

        self.transfer(&mut buffer[..n]);
        Delay.delay_ms(15u32);
        Ok(core::str::from_utf8(&buffer[..n]).unwrap())
    }
}
