use tock_registers::{interfaces::Readable, register_bitfields};

register_bitfields! {u32,
    ICC_IAR1_EL1 [
        /// The INTID of the signaled interrupt.
        INTID OFFSET(0) NUMBITS(24) []
    ]
}

pub struct Reg;

impl Readable for Reg {
    type T = u32;
    type R = ICC_IAR1_EL1::Register;

    sys_coproc_read_raw!(u32, "ICC_IAR1_EL1", "x");
}

impl Reg {
    #[inline]
    pub fn get_pending_interrupt(&self) -> u32 {
        self.read(ICC_IAR1_EL1::INTID)
    }
}

pub static ICC_IAR1_EL1: Reg = Reg {};
