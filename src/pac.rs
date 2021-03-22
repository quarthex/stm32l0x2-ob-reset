use core::ops::Deref;

pub mod ob {
    use core::marker::PhantomData;
    use stm32l0xx_hal::pac::FLASH;
    use volatile_register::RW;

    pub struct Reg<T> {
        inner: RW<u32>,
        phantom: PhantomData<T>,
    }

    impl<T> Reg<T> {
        pub fn read(&self) -> u32 {
            self.inner.read()
        }

        pub fn read_u16(&self) -> Option<u16> {
            let word = self.read();
            if (word >> 16) == (!word & 0xffff) {
                Some(word as u16)
            } else {
                None
            }
        }

        fn wait_eop(flash: &FLASH) -> Result<(), ()> {
            while flash.sr.read().bsy().is_active() { /* wait */ }
            if flash.sr.read().eop().is_event() {
                flash.sr.modify(|_, w| w.eop().clear_bit());
                Ok(())
            } else {
                Err(())
            }
        }

        pub unsafe fn write(&self, value: u32, flash: &FLASH) -> Result<(), ()> {
            self.inner.write(value);
            Self::wait_eop(flash)
        }

        pub unsafe fn write_u16(&self, value: u16, flash: &FLASH) -> Result<(), ()> {
            let value = u32::from(value);
            self.write((!value << 16) | (value & 0xffff), flash)
        }

        pub unsafe fn erase(&self, flash: &FLASH) -> Result<(), ()> {
            flash.pecr.modify(|_, w| w.erase().erase());
            self.inner.write(0);
            let result = Self::wait_eop(flash);
            flash.pecr.modify(|_, w| w.erase().no_erase());
            result
        }
    }

    /// Least significant halfword of the OPTR register
    pub struct OPTRL;
    /// Most significant halfword of the OPTR register
    pub struct OPTRH;
    /// Least significant halfword of the WRPROT1 register
    pub struct WRPROT1L;
    /// Most significant halfword of the WRPROT1 register
    pub struct WRPROT1H;
    /// Least significant halfword of the WRPROT2 register
    pub struct WRPROT2L;

    /// Register block
    #[repr(C)]
    pub struct RegisterBlock {
        /// 0x00 − Least significant halfword of the OPTR register
        pub optrl: Reg<OPTRL>,
        /// 0x04 − Most significant halfword of the OPTR register
        pub optrh: Reg<OPTRH>,
        /// 0x08 − Least significant halfword of the WRPROT1 register
        pub wrprot1l: Reg<WRPROT1L>,
        /// 0x0c − Most significant halfword of the WRPROT1 register
        pub wrprot1h: Reg<WRPROT1H>,
        /// 0x10 − Least significant halfword of the WRPROT2 register
        pub wrprot2l: Reg<WRPROT2L>,
    }
}

/// Low-level Option bytes interface
pub struct OB;

impl OB {
    pub const fn ptr() -> *const ob::RegisterBlock {
        0x1ff8_0000 as *const ob::RegisterBlock
    }
}

impl Deref for OB {
    type Target = ob::RegisterBlock;

    fn deref(&self) -> &Self::Target {
        unsafe { &*Self::ptr() as &Self::Target }
    }
}
