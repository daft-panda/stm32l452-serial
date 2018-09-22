//!
//! ---
//!
#![no_main]
#![no_std]

extern crate cortex_m;
#[macro_use]
extern crate cortex_m_rt as rt;
extern crate cortex_m_semihosting as sh;
extern crate ftl_tlc;
extern crate panic_semihosting;
#[macro_use]
extern crate stm32l4;

use rt::ExceptionFrame;
use stm32l4::stm32l4x2::Interrupt;
use cortex_m::asm;
use core::fmt::Write;
use core::ptr;
use core::cell::{RefCell};

entry!(main);

fn main() -> ! {
    unsafe {
        let mut x = 0xE000_E008 as u32;
        let y = &mut x as *mut u32;
        // Disable Cortex-M4 write buffer for debugging (DISDEFWBUF)
        ptr::write_volatile(y, 0b10);
    }

    let p = stm32l4::stm32l4x2::Peripherals::take().unwrap();

    let mut clocks = ftl_tlc::clocks::Clocks {
        rcc: &p.RCC,
        pwr: &p.PWR,
        flash: RefCell::new(p.FLASH)
    };

    clocks.init();

    let tim6 = p.TIM6;
    let gpiob = p.GPIOB;

    let cmp = cortex_m::Peripherals::take().unwrap();
    let mut nvic = cmp.NVIC;

    nvic.enable(Interrupt::EXTI0);
    nvic.enable(Interrupt::TIM6_DACUNDER);

    // Configure the pin PB13 as an output pin
    gpiob.moder.modify(|_, w| w.moder13().output());

    // Configure USART2 pins A2,A3
    let gpioa = p.GPIOA;
    gpioa.moder.write(|w| w.moder2().alternate().moder3().alternate());
    gpioa.ospeedr.write(|w| w.ospeedr2().very_high_speed().ospeedr3().very_high_speed());
    gpioa.pupdr.write(|w| w.pupdr2().floating().pupdr3().floating());
    gpioa.afrl.write(|w| w.afrl2().af7().afrl3().af7());

    // Configure TIM7 for periodic timeouts gpiod
    let apb1_freq: u32 = 48_000_000;
    let frequency: u32 = 1;
    let ratio = apb1_freq / frequency;
    let psc = ((ratio - 1) / u16::max_value() as u32) as u16;
    unsafe {
        tim6.psc.write(|w| w.psc().bits(psc));
        let arr = (ratio / (psc + 1) as u32) as u16;
        tim6.arr.write(|w| w.arr().bits(arr));
        tim6.cr1.write(|w| w.opm().clear_bit());
        tim6.dier.write(|w| w.uie().set_bit());
    }

    // Start the timer
    tim6.cr1.modify(|_, w| w.cen().set_bit());

    let mut uart = ftl_tlc::uart::UART {
        usart2: &p.USART2
    };

    uart.init();

    writeln!(uart, "Hello, world!\r").unwrap();

    loop {
        asm::wfi();
    }
}


interrupt!(TIM6_DACUNDER, tim6, state: bool = false);

fn tim6(state: &mut bool) {
    let mut ls: bool = *state;
    ls = !ls;

    unsafe {

        let p = stm32l4::stm32l4x2::Peripherals::steal();

        let tim6 = p.TIM6;
        let gpiob = p.GPIOB;

        if ls {
            gpiob.bsrr.write(|w| w.br13().reset());
        } else {
            gpiob.bsrr.write(|w| w.bs13().set());
        }

        // Clear the update event flag
        tim6.sr.modify(|_, w| w.uif().clear_bit());
    }

    *state = ls;
}

exception!(HardFault, hard_fault);

fn hard_fault(ef: &ExceptionFrame) -> ! {
    asm::bkpt();
    panic!("HardFault at {:#?}", ef);
}

exception!(*, default_handler);

fn default_handler(irqn: i16) {
    asm::bkpt();
    panic!("Unhandled exception (IRQn = {})", irqn);
}
