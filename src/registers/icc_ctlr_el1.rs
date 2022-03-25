use tock_registers::{
    interfaces::{Readable, Writeable},
    register_bitfields,
};

register_bitfields! {u32,
    pub ICC_CTLR_EL1 [
        /// Common Binary Point Register.
        CNPR OFFSET(0) NUMBITS(1) [],
        /// EOI mode for the current Security state.
        EOImode OFFSET(1) NUMBITS(1) []
    ]
}

pub struct Reg;

impl Readable for Reg {
    type T = u32;
    type R = ICC_CTLR_EL1::Register;

    sys_coproc_read_raw!(u32, "ICC_CTLR_EL1", "x");
}

impl Writeable for Reg {
    type T = u32;
    type R = ICC_CTLR_EL1::Register;

    sys_coproc_write_raw!(u32, "ICC_CTLR_EL1", "x");
}

impl Reg {
    #[inline]
    pub fn get_eoi_mode(&self) -> bool {
        self.read(ICC_CTLR_EL1::EOImode) == 1
    }

    #[inline]
    pub fn set_eoi_mode(&self, eoi_mode: bool) {
        self.write(ICC_CTLR_EL1::EOImode.val(eoi_mode as u32));
    }
}

pub static ICC_CTLR_EL1: Reg = Reg {};
