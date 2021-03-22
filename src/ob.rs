use crate::pac;
use cortex_m::interrupt;
use stm32l0xx_hal::pac::FLASH;

/// High-level Option bytes interface
pub struct OB {
    inner: pac::OB,
}

impl OB {
    pub const fn new(ob: pac::OB) -> Self {
        Self { inner: ob }
    }

    fn unlock_pecr<T, F: FnOnce(&mut FLASH) -> T>(flash: &mut FLASH, f: F) -> T {
        if flash.pecr.read().pelock().is_locked() {
            for word in &[0x89ab_cdef, 0x0203_0405] {
                flash.pekeyr.write(|w| unsafe { w.bits(*word) });
            }
            let result = f(flash);
            flash.pecr.modify(|_, w| w.pelock().set_bit());
            result
        } else {
            f(flash)
        }
    }

    fn unlock_optr<T, F: FnOnce(&mut FLASH) -> T>(flash: &mut FLASH, f: F) -> T {
        if flash.pecr.read().optlock().is_locked() {
            Self::unlock_pecr(flash, |flash| {
                for word in &[0xfbea_d9c8, 0x2425_2627] {
                    flash.optkeyr.write(|w| unsafe { w.bits(*word) });
                }
                let result = f(flash);
                flash.pecr.modify(|_, w| w.optlock().set_bit());
                result
            })
        } else {
            f(flash)
        }
    }

    fn modify_option_byte<T, F: FnOnce(&mut FLASH) -> T>(flash: &mut FLASH, f: F) -> T {
        interrupt::free(|_critical_section| {
            while flash.sr.read().bsy().is_active() { /* wait */ }
            Self::unlock_optr(flash, f)
        })
    }

    /// Factory value of the OPTR register
    pub const OPTR_RESET_VALUE: u32 = 0x8070_00AA;
    const OPTRL_RESET_VALUE: u16 = Self::OPTR_RESET_VALUE as u16;
    const OPTRH_RESET_VALUE: u16 = (Self::OPTR_RESET_VALUE >> 16) as u16;

    pub fn reset_optr(&self, flash: &mut FLASH) -> Result<(), ()> {
        let optrl = self.inner.optrl.read_u16();
        let optrh = self.inner.optrh.read_u16();

        if let Some(Self::OPTRL_RESET_VALUE) = optrl {
            if let Some(Self::OPTRH_RESET_VALUE) = optrh {
                return Ok(());
            }
        }

        Self::modify_option_byte(flash, |flash| unsafe {
            if !matches!(optrl, Some(Self::OPTRL_RESET_VALUE)) {
                self.inner.optrl.erase(flash)?; // Is it necessary?
                self.inner.optrl.write_u16(Self::OPTRL_RESET_VALUE, flash)?;
            }
            if !matches!(optrh, Some(Self::OPTRH_RESET_VALUE)) {
                self.inner.optrh.erase(flash)?; // Is it necessary?
                self.inner.optrh.write_u16(Self::OPTRH_RESET_VALUE, flash)?;
            }
            Ok(())
        })
    }

    /// Factory value of the WRPROT1 register
    pub const WRPROT1_RESET_VALUE: u32 = 0x0000_0000;
    const WRPROT1L_RESET_VALUE: u16 = Self::WRPROT1_RESET_VALUE as u16;
    const WRPROT1H_RESET_VALUE: u16 = (Self::WRPROT1_RESET_VALUE >> 16) as u16;

    pub fn reset_wrprot1(&self, flash: &mut FLASH) -> Result<(), ()> {
        let wrprot1l = self.inner.wrprot1l.read_u16();
        let wrprot1h = self.inner.wrprot1h.read_u16();

        if let Some(Self::WRPROT1L_RESET_VALUE) = wrprot1l {
            if let Some(Self::WRPROT1H_RESET_VALUE) = wrprot1h {
                return Ok(());
            }
        }

        Self::modify_option_byte(flash, |flash| unsafe {
            if !matches!(wrprot1l, Some(Self::WRPROT1L_RESET_VALUE)) {
                self.inner.wrprot1l.erase(flash)?; // Is it necessary?
                self.inner.wrprot1l.write_u16(0, flash)?;
            }
            if !matches!(wrprot1h, Some(Self::WRPROT1H_RESET_VALUE)) {
                self.inner.wrprot1h.erase(flash)?; // Is it necessary?
                self.inner.wrprot1h.write_u16(0, flash)?;
            }
            Ok(())
        })
    }

    /// Factory value of the WRPROT2 register
    pub const WRPROT2_RESET_VALUE: u16 = 0;

    pub fn reset_wrprot2(&self, flash: &mut FLASH) -> Result<(), ()> {
        if let Some(Self::WRPROT2_RESET_VALUE) = self.inner.wrprot2l.read_u16() {
            Ok(())
        } else {
            Self::modify_option_byte(flash, |flash| unsafe {
                self.inner.wrprot2l.erase(flash)?; // Is it necessary?
                self.inner
                    .wrprot2l
                    .write_u16(Self::WRPROT2_RESET_VALUE, flash)
            })
        }
    }
}
