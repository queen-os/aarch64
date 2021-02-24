use register::{cpu::RegisterReadWrite, register_bitfields};

register_bitfields! {u32,
    ICC_IGRPEN1_EL1 [
        Enable OFFSET(0) NUMBITS(1) []
    ]
}

pub struct Reg;

impl RegisterReadWrite<u32, ICC_IGRPEN1_EL1::Register> for Reg {
    sys_coproc_read_raw!(u32, "ICC_IGRPEN1_EL1", "x");
    sys_coproc_write_raw!(u32, "ICC_IGRPEN1_EL1", "x");
}

impl Reg {
    #[inline]
    pub fn get_enable(&self) -> bool {
        self.read(ICC_IGRPEN1_EL1::Enable) == 1
    }
    #[inline]
    pub fn set_enable(&self, enable: bool) {
        self.write(ICC_IGRPEN1_EL1::Enable.val(enable as u32));
    }
}

pub static ICC_IGRPEN1_EL1: Reg = Reg {};
