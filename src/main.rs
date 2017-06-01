#![feature(used)]
#![no_std]

// version = "0.2.0", default-features = false
extern crate cast;
#[macro_use]
extern crate cortex_m;
extern crate cortex_m_rt;
extern crate stm32f446;

use core::u16;

use cast::{u16, u32};
use cortex_m::asm;
use stm32f446::{GPIOA, GPIOB, RCC, TIM3, ADC1, CAN1};

mod frequency {
    /// Frequency of APB1 bus (TIM3 is connected to this bus)
    pub const APB1: u32 = 8_000_000;
}

/// Timer frequency
const FREQUENCY: u32 = 1;

#[inline(never)]
fn main() {
    // Critical section, this closure is non-preemptable
    cortex_m::interrupt::free(
        |cs| unsafe {

            
            // INITIALIZATION PHASE
            // Exclusive access to the peripherals
            let gpioa = GPIOA.borrow(cs);
            let gpiob = GPIOB.borrow(cs);
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

            hprintln!("ratio {:?} psc {:?} arr {:?}", ratio, psc, arr);


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
                //w.swstart().bits(1); //Start sampling
                w
            });

            //Cant write adon and swstart at the same timen
            adc1.cr2.write(|w| {
                w.cont().bits(1); //Continuous mode
                w.adon().bits(1); //ADC on
                w.swstart().bits(1); //Start sampling
                w
            });

            //TODO: change to a modify op
            //adc1.cr2.modify(|w| w.swstart().bits(1));

            //Parameters for linear feedback from ADC to blink timer
            let Xcoeff = 595/33;
            let Yint = 459500/33;
            
            // APPLICATION LOGIC
            let mut state = false;
            loop {
                // Wait for an update event
                while tim3.sr.read().uif().bits() == 0 {}

                //Calculate and write new arr for timer 3
                let lvl = adc1.dr.read().data().bits(); //Read the ADC level
                let arr = Xcoeff * lvl - Yint;

                tim3.arr.write(|w| w.arr_l().bits(arr));

                // Clear the update event flag
                tim3.sr.write(|w| w.uif().bits(0));

                // Toggle the state
                state = !state;

                //hprintln!("ADC {:?} \r", lvl);
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

fn CAN_init(){
    cortex_m::interrupt::free(
    |cs| unsafe {
        // Initializes the CAN bus on Nucleo pins D14 and D15
        let gpiob = GPIOB.borrow(cs);
        let can1 = CAN1.borrow(cs); // Not sure about this...

        // Set board D14 (STM PB8) and board D15 (STM PB9)
        // to CAN1_Rx and CAN1_Tx, respectively.

        // Set pins to use alternate function
        gpiob.moder.write(|w|{
            w.moder8().bits(2);
            w.moder9().bits(2);
            w
        });

        // Set each to AF9 (1001) using AFRHb
        gpiob.afrh.write(|w|{
            w.afrh9().bits(5);
            w.afrh8().bits(5);
            w
        });

        // Enter initialization mode. Set INRQ on CAN_MCR
        can1.mcr.write(|w| w.inrq().bits(1));

        // Wait for INAK bit on CAN_MSR for confirmation
        while can1.msr.read().inak().bits() == 0 {}


        // Set up bit timing on CAN_BTR (I think defaults might be ok)
        //can1.btr.write(|w| )

        // Set up CAN options on CAN_MCR (but i dont think I need any)


        // Go to normal mode. Clear INRQ an CAN_MCR
        can1.mcr.write(|w| w.inrq().bits(1));

        // Hardware listens for sync (11 recessive bits)
        // Hardware confirms sync by clearing INAK on CAN_MSR
        while can1.msr.read().inak().bits() == 0 {}

        hprintln!("CAN"); //Ready msg

        //Send out a test CAN message
        //TODO: make CAN transmit into its own fn

        let ident = 8;
        let dataLength = 1;
        let data = 170;

        //Select an empty outbox

        //Set identifier
        can1.ti0r.write(|w| w.stid().bits(ident));

        //Set DLC
        can1.tdt0r.write(|w| w.dlc().bits(dataLength));

        //Set data
        can1.tdl0r.write(|w| w.data0().bits(data));

        //Set TXRQ
        can1.ti0r.write(|w| {
            w.stid().bits(ident);
            w.txrq().bits(1);
            w
        });
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
