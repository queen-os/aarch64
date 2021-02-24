use register::{cpu::RegisterReadWrite, register_bitfields};

register_bitfields! {u32,
    pub ICC_PMR_EL1 [
        /// The priority mask level for the CPU interface. If the priority of an interrupt is higher than the value
        /// indicated by this field, the interface signals the interrupt to the PE.
        Priority OFFSET(0) NUMBITS(8) []
    ]
}

pub struct Reg;

impl RegisterReadWrite<u32, ICC_PMR_EL1::Register> for Reg {
    sys_coproc_read_raw!(u32, "ICC_PMR_EL1", "x");
    sys_coproc_write_raw!(u32, "ICC_PMR_EL1", "x");
}

impl Reg {
    #[inline]
    pub fn get_priority(&self) -> u32 {
        self.read(ICC_PMR_EL1::Priority)
    }

    #[inline]
    pub fn set_priority(&self, priority: u32) {
        self.write(ICC_PMR_EL1::Priority.val(priority));
    }
}

pub static ICC_PMR_EL1: Reg = Reg {};
