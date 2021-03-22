#![no_std]
#![no_main]

mod ob;
mod pac;

use crate::ob::OB;
use cortex_m_rt::entry;
use panic_halt as _; // freeze on panic
use stm32l0xx_hal::pac::Peripherals;

#[entry]
fn main() -> ! {
    // Get the STM32L0 peripherals
    let mut periphs = Peripherals::take().unwrap();

    // Create the option bytes interface
    let ob = OB::new(pac::OB);

    loop {
        // Reset OPTR (Option bytes register)
        ob.reset_optr(&mut periphs.FLASH).unwrap();

        // Reset WRPROT1 (Write protection register 1)
        ob.reset_wrprot1(&mut periphs.FLASH).unwrap();

        // Reset WRPROT1 (Write protection register 2)
        ob.reset_wrprot2(&mut periphs.FLASH).unwrap();
    }
}
