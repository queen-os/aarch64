use tock_registers::{
    interfaces::{Readable, Writeable},
    register_bitfields,
};

register_bitfields! {u32,
    /// A PE writes to this register to inform the CPU interface that it has completed the processing of the
    /// specified Group 1 interrupt.
    pub ICC_EOIR1_EL1 [
        /// The INTID from the corresponding ICC_IAR1_EL1 access.
        INTID OFFSET(0) NUMBITS(24) []
    ]
}

pub struct Reg;

impl Readable for Reg {
    type T = u32;
    type R = ICC_EOIR1_EL1::Register;

    sys_coproc_read_raw!(u32, "ICC_EOIR1_EL1", "x");
}

impl Writeable for Reg {
    type T = u32;
    type R = ICC_EOIR1_EL1::Register;

    sys_coproc_write_raw!(u32, "ICC_EOIR1_EL1", "x");
}

impl Reg {
    pub fn mark_completed(&self, irq_num: u32) {
        self.write(ICC_EOIR1_EL1::INTID.val(irq_num));
    }
}

pub static ICC_EOIR1_EL1: Reg = Reg {};
