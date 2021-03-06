#![feature(used)]
#![no_std]

// version = "0.2.0", default-features = false
extern crate cast;
#[macro_use]
extern crate cortex_m;
extern crate cortex_m_rt;
extern crate stm32f446;

use core::u16;
use core::cmp::min;
use cast::{u16, u32};
use cortex_m::asm;
use stm32f446::{GPIOA, GPIOB, RCC, TIM3, ADC1, CAN1};

mod frequency {
    /// Frequency of APB1 bus (TIM3 is connected to this bus)
    pub const APB1: u32 = 16_000_000;
}

/// Timer frequency
const FREQUENCY: u32 = 1;

#[inline(never)]
fn main() {
    // Critical section, this closure is non-preemptable
    cortex_m::interrupt::free(
        |cs| unsafe {

            hprintln!("Hello");
            // INITIALIZATION PHASE
            // Exclusive access to the peripherals
            let gpioa = GPIOA.borrow(cs);
            //let gpiob = GPIOB.borrow(cs);
            let rcc = RCC.borrow(cs);
            let tim3 = TIM3.borrow(cs);
            let adc1 = ADC1.borrow(cs);

            // Power up the relevant peripherals
            rcc.ahb1enr.write(|w|{
                 w.gpioaen().bits(1);
                 w.gpioben().bits(1);
                 w
            });
            rcc.apb1enr.write(|w|{
                w.tim3en().bits(1);
                w.can1en().bits(1);
                w
            });
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


            // Set up ADCn
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
            let x_coeff = 595/33;
            let y_int = 459500/33;
            
            //Run CAN initializer
            canbus_init();

            // APPLICATION LOGIC
            let mut state = false;
            loop {
                // Wait for an update event
                while tim3.sr.read().uif().bits() == 0 {}

                //Calculate and write new arr for timer 3
                let lvl = adc1.dr.read().data().bits(); //Read the ADC level
                let arr = x_coeff * lvl - y_int;

                tim3.arr.write(|w| w.arr_l().bits(arr));

                // Clear the update event flag
                tim3.sr.write(|w| w.uif().bits(0));

                // Toggle the state
                state = !state;

                // Blink the LED
                if state {
                    gpioa.bsrr.write(|w| w.bs5().bits(1));
                    canbus_tx(222, &[1, 4, 8]);
                } else {
                    gpioa.bsrr.write(|w| w.br5().bits(1));
                    canbus_tx(222, &[2, 5, 9]);
                }

                
            }
        },
    );
}

fn canbus_init(){
    cortex_m::interrupt::free(
    |cs| unsafe {
        // Initializes the CAN bus on Nucleo pins D14 and D15
        let gpiob = GPIOB.borrow(cs);
        let can1 = CAN1.borrow(cs); 

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
            w.afrh9().bits(9);
            w.afrh8().bits(9);
            w
        });

        // Enter initialization mode. Set INRQ on CAN_MCR
        can1.mcr.write(|w| {
            w.inrq().bits(1);
            w.sleep().bits(0);
            w
        });

        // Wait for INAK bit on CAN_MSR for confirmation
        while can1.msr.read().inak().bits() == 0 {}
        
        // Set up CAN timing (peripheral timer is 16 MHz) 
        can1.btr.write(|w|{ 
            //w.lbkm().bits(1); //loopback
            //w.silm().bits(1); //silentmode

            //Set for 500kbps rate
            //w.brp().bits(1);
            //w.ts1().bits(12);
            //w.ts2().bits(1);


            w.brp().bits(99);
            w.ts1().bits(12);
            w.ts2().bits(1);
            w
        }); 

        // Go to normal mode. Clear INRQ an CAN_MCR
        can1.mcr.write(|w|{
             w.inrq().bits(0);
             w.sleep().bits(0);
             w
        });
       
        // Hardware listens for sync (11 recessive bits)
        // Hardware confirms sync by clearing INAK on CAN_MSR
        while can1.msr.read().inak().bits() == 1 {}    

    },
    );
}

fn canbus_tx(ident: u16, data: &[u8]){
    cortex_m::interrupt::free(
    |cs| unsafe {
        //Send out a test CAN message
        
        let can1 = CAN1.borrow(cs); 
        let data_length = min(data.len(), 8) as u8;
        //TODO: Check that data is not too long.
        

        //Select an empty outbox
        if can1.tsr.read().tme0().bits() == 0 {
           
        }
        else {
            
        }
        
        //Set DLC
        can1.tdt0r.write(|w| w.dlc().bits(data_length));

        //Set data
        can1.tdl0r.write(|w| {
            w.data0().bits(*data.get(0).unwrap_or(&0));
            w.data1().bits(*data.get(1).unwrap_or(&0));
            w.data2().bits(*data.get(2).unwrap_or(&0));
            w.data3().bits(*data.get(3).unwrap_or(&0));
            w
        });
        can1.tdh0r.write(|w| {
            w.data4().bits(*data.get(4).unwrap_or(&0));
            w.data5().bits(*data.get(5).unwrap_or(&0));
            w.data6().bits(*data.get(6).unwrap_or(&0));
            w.data7().bits(*data.get(7).unwrap_or(&0));
            w
        });

        //Set TXRQ
        can1.ti0r.write(|w| {
            w.stid().bits(ident);
            w.txrq().bits(1);
            w
        });

        //Wait for request completed
        while can1.tsr.read().rqcp0().bits() == 0 {}

    });
}
    


// This part is the same as before
#[allow(dead_code)]
#[used]
#[link_section = ".rodata.interrupts"]
static INTERRUPTS: [extern "C" fn(); 240] = [default_handler; 240];

extern "C" fn default_handler() {
    asm::bkpt();
}
