pub use cortex_a::asm::*;
use cortex_a::regs::RegisterReadOnly;

/// Returns the current stack pointer.
#[inline]
pub fn sp() -> *const u8 {
    let ptr: usize;
    unsafe {
        asm!("mov {0}, sp", out(reg) ptr, options(pure, nomem, nostack));
    }
    ptr as *const u8
}

/// # Safety
///
/// Returns the current point counter.
#[inline]
pub unsafe fn pc() -> usize {
    let pc: usize;
    asm!("adr {}, .", out(reg) pc, options(pure, nomem, nostack));
    pc
}

/// CPU id
#[inline]
pub fn cpuid() -> usize {
    (cortex_a::regs::MPIDR_EL1.get() & 3) as usize
}

/// # Safety
///
/// The halt function stops the processor until the next interrupt arrives
#[inline]
pub unsafe fn halt() {
    asm!("wfi", options(nomem, nostack));
}
