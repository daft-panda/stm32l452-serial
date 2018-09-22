extern crate cortex_m;
extern crate stm32l4;

use core::cell::{RefCell, RefMut};
use core::ptr;
use cortex_m::asm;

pub struct Clocks<'a> {
    pub rcc: &'a stm32l4::stm32l4x2::RCC,
    pub pwr: &'a stm32l4::stm32l4x2::PWR,
    pub flash: RefCell<stm32l4::stm32l4x2::FLASH>,
}

impl<'a> Clocks<'a> {
    pub fn init(&mut self) {
        // Power up the relevant peripherals
        self.rcc.apb1enr1.write(|w| w.pwren().set_bit());

        while self.rcc.apb1enr1.read().pwren().bit_is_clear() {}

        self.rcc.apb2enr.write(|w| w.syscfgen().set_bit());
        while self.rcc.apb2enr.read().syscfgen().bit_is_clear() {}

        self.rcc.ahb2enr.write(|w| w.gpioaen().set_bit());
        while self.rcc.ahb2enr.read().gpioaen().bit_is_clear() {};

        // Configure LSE drive
        self.pwr.cr1.write(|w| w.dbp().set_bit());

        // Wait for write protection to be disabled
        while self.pwr.cr1.read().dbp().bit_is_clear() {};

        unsafe {
            self.rcc.bdcr.write(|w| w.lsedrv().bits(0));
        }

        // Note: clock configuration implemented based on STM32MX calculator
        // Set MSI as the main clock source, at 48MHz
        unsafe {
            self.rcc.cr.write(|w| w
                // Enable MSI clock
                .msion().set_bit()
                // Disable PLL, don't need it
                .pllon().clear_bit());

            // Wait for MSI clock to become ready
            while self.rcc.cr.read().msirdy().bit_is_clear() {}

            // Configure flash latency
            let vos = self.pwr.cr1.read().vos().bits();
            let msirange: u8 = 0xb;
            let mut latency: u8 = 0;

            if vos == 1 {
                if msirange > 8 {
                    latency = 2;
                } else {
                    latency = 1
                }
            } else {
                if msirange > 8 {
                    latency = 3;
                } else {
                    if msirange == 8 {
                        latency = 2;
                    } else {
                        latency = 1;
                    }
                }
            }

            let flash = self.flash.borrow_mut();
            flash.acr.write(|w| w.latency().bits(latency));
            if flash.acr.read().latency().bits() != latency {
                panic!("at the disco")
            }
            // self.flash.deref_mut();

            // Range will be set in the MSI range
            self.rcc.cr.write(|w| w.msirgsel().set_bit());

            // Sets MSI at about 48MHz
            self.rcc.cr.write(|w| w.msirange().bits(msirange));

            // Activate LSE clock, used for MSI calibration
            self.rcc.bdcr.write(|w| w
                .lseon().set_bit());

            // Wait for LSE to become ready
            while self.rcc.bdcr.read().lserdy().bit_is_clear() {}

            self.rcc.cfgr.write(|w| w
                // MSI as system clock source
                .sw().bits(0)
                // No AHB prescaling
                .hpre().bits(0)
                // No APB1 prescaling
                .ppre1().bits(0)
                // No APB2 prescaling
                .ppre2().bits(0));

            /*let mut x = 0x40021000 as u32;
            let y = &mut x as *mut u32;
            ptr::write_volatile(y, 0xbf);*/

            self.rcc.cr.modify(|_, w| w
                // Enable LSE based auto-calibration
                .msipllen().set_bit());
        }

        self.rcc.ahb2enr.modify(|_, w| w
            .gpioaen().set_bit()
            .gpioben().set_bit());

        self.rcc.apb1enr1.write(|w| w
            .pwren().set_bit()
            .tim6en().set_bit()
            .usart2en().set_bit());
    }
}