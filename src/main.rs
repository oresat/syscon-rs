#![feature(used)]
#![no_std]

// version = "0.2.0", default-features = false
extern crate cast;
#[macro_use]
extern crate cortex_m;
extern crate cortex_m_rt;
extern crate stm32f411xx;

use core::u16;

use cast::{u16, u32};
use cortex_m::asm;
use stm32f411xx::{GPIOA, RCC, TIM3, ADC1};

mod frequency {
    /// Frequency of APB1 bus (TIM3 is connected to this bus)
    pub const APB1: u32 = 8_000_000;
}

/// Timer frequency
const FREQUENCY: u32 = 1;

#[inline(never)]
fn main() {
    // Critical section, this closure is non-preemptable
    hprintln!("htaoensuhotaensu;");
    cortex_m::interrupt::free(
        |cs| unsafe {

            
            // INITIALIZATION PHASE
            // Exclusive access to the peripherals
            let gpioa = GPIOA.borrow(cs);
            let rcc = RCC.borrow(cs);
            let tim3 = TIM3.borrow(cs);
            let adc1 = ADC1.borrow(cs);

            // Power up the relevant peripherals
            rcc.ahb1enr.write(|w| w.gpioaen().bits(1));
            rcc.apb1enr.write(|w| w.tim3en().bits(1));
            rcc.apb2enr.write(|w| w.adc1en().bits(1));

            // Configure the pin PA5 as an output pin and PA0 as AIN
            gpioa.moder.write(|w| {
                w.moder5().bits(1);
                w.moder0().bits(3); 
                w //"Return"
            });
            

            // Configure TIM3 for periodic timeouts
            let ratio = frequency::APB1 / FREQUENCY;
            let psc = u16((ratio - 1) / u32(u16::MAX)).unwrap();
            tim3.psc.write(|w| w.psc().bits(psc));
            let arr = u16(ratio / u32(psc + 1)).unwrap();
            tim3.arr.write(|w| w.arr_l().bits(arr));


            // Start the timer
            tim3.cr1.write(|w| w.cen().bits(1));


            // Set up ADC

            // We only want one conversion
            adc1.sqr1.write(|w| w.l().bits(0));
            // The first conversion should look at PA0
            adc1.sqr3.write(|w| w.sq1().bits(0));

            adc1.cr2.write(|w| {
                w.cont().bits(1); //Continuous mode
                w.adon().bits(1); //ADC on
                w.swstart().bits(1); //Start sampling
                w
            });

            // APPLICATION LOGIC
            let mut state = false;
            loop {
                // Wait for an update event
                while tim3.sr.read().uif().bits() == 0 {}

                // Clear the update event flag
                tim3.sr.write(|w| w.uif().bits(0));

                // Toggle the state
                state = !state;

                // Blink the LED
                if state {
                    gpioa.bsrr.write(|w| w.bs5().bits(1));
                } else {
                    gpioa.bsrr.write(|w| w.br5().bits(1));
                }
            }
        },
    );
}

// This part is the same as before
#[allow(dead_code)]
#[used]
#[link_section = ".rodata.interrupts"]
static INTERRUPTS: [extern "C" fn(); 240] = [default_handler; 240];

extern "C" fn default_handler() {
    asm::bkpt();
}
