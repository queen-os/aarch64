#[macro_use]
mod macros;

mod ctr_el0;
mod icc_ctlr_el1;
mod icc_eoir1_el1;
mod icc_iar1_el1;
mod icc_igrpen1_el1;
mod icc_pmr_el1;
mod icc_sre_el1;
mod ttbr0_el1;
mod ttbr1_el1;

pub use tock_registers::interfaces::{Readable, Writeable, ReadWriteable};
pub use cortex_a::registers::*;
pub use ctr_el0::CTR_EL0;
pub use icc_ctlr_el1::ICC_CTLR_EL1;
pub use icc_eoir1_el1::ICC_EOIR1_EL1;
pub use icc_iar1_el1::ICC_IAR1_EL1;
pub use icc_igrpen1_el1::ICC_IGRPEN1_EL1;
pub use icc_pmr_el1::ICC_PMR_EL1;
pub use icc_sre_el1::ICC_SRE_EL1;
pub use ttbr0_el1::TTBR0_EL1;
pub use ttbr1_el1::TTBR1_EL1;
