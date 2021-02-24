#[macro_use]
mod macros;

mod ctr_el0;
mod icc_ctlr_el1;
mod icc_iar0_el1;
mod icc_iar1_el1;
mod icc_igrpen1_el1;
mod icc_pmr_el1;
mod icc_sre_el1;
mod mair_el1;
mod ttbr0_el1;
mod ttbr1_el1;

pub use cortex_a::regs::*;
pub use ctr_el0::CTR_EL0;
pub use icc_ctlr_el1::ICC_CTLR_EL1;
pub use icc_iar0_el1::ICC_IAR0_EL1;
pub use icc_iar1_el1::ICC_IAR1_EL1;
pub use icc_igrpen1_el1::ICC_IGRPEN1_EL1;
pub use icc_pmr_el1::ICC_PMR_EL1;
pub use icc_sre_el1::ICC_SRE_EL1;
pub use mair_el1::{MAIR_ATTR, MAIR_EL1};
pub use ttbr0_el1::TTBR0_EL1;
pub use ttbr1_el1::TTBR1_EL1;
