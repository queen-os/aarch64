#[macro_use]
mod macros;

mod ctr_el0;
mod mair_el1;
mod ttbr0_el1;
mod ttbr1_el1;

pub use cortex_a::regs::*;
pub use ctr_el0::CTR_EL0;
pub use mair_el1::{MAIR_ATTR, MAIR_EL1};
pub use ttbr0_el1::TTBR0_EL1;
pub use ttbr1_el1::TTBR1_EL1;
