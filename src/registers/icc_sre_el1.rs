use register::{cpu::RegisterReadWrite, register_bitfields};

register_bitfields! {u32,
    pub ICC_SRE_EL1 [
        /// System Register Enable.
        SRE OFFSET(0) NUMBITS(1) [],
        /// Disable FIQ bypass.
        DFB OFFSET(1) NUMBITS(1) [],
        /// Disable IRQ bypass.
        DIB OFFSET(2) NUMBITS(1) []
    ]
}

pub struct Reg;

impl RegisterReadWrite<u32, ICC_SRE_EL1::Register> for Reg {
    sys_coproc_read_raw!(u32, "ICC_SRE_EL1", "x");
    sys_coproc_write_raw!(u32, "ICC_SRE_EL1", "x");
}

impl Reg {
    #[inline]
    pub fn get_sre(&self) -> bool {
        self.read(ICC_SRE_EL1::SRE) == 1
    }

    #[inline]
    pub fn set_sre(&self, sre: bool) {
        self.write(ICC_SRE_EL1::SRE.val(sre as u32));
    }
}

pub static ICC_SRE_EL1: Reg = Reg {};
