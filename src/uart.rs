extern crate stm32l4;

use core::{fmt, slice};

pub struct UART<'a> {
    pub usart2: &'a stm32l4::stm32l4x2::USART2
}

impl<'a> UART<'a> {
    fn baud_rate_to_brr(pclk:u32, baudrate:u32) -> u16 {
        let div = pclk as f32/ baudrate as f32;

        return div as u16;
    }

    pub fn init(&mut self) {
        let brr:u32 = UART::baud_rate_to_brr(48_000_000, 115200) as u32;
        unsafe {
            self.usart2.brr.write(|w| w.bits(0x1A1));
        }

        self.start();
    }

    fn start(&mut self) {
        self.usart2.cr1.write(|w| w
            .ue().set_bit()
            .te().set_bit()
            .re().set_bit());
    }

    pub fn write_all(&mut self, buffer: &[u8]) -> Result<(), ()> {
        for ch in buffer {
            while self.usart2.isr.read().txe().bit_is_clear() {}

            self.usart2.tdr.write(|w| unsafe {w.bits(*ch as u32)});
        }

        while self.usart2.isr.read().tc().bit_is_clear() {}

        Ok(())
    }
}

impl<'a> fmt::Write for UART<'a> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_all(s.as_bytes()).map_err(|_| fmt::Error)
    }
}