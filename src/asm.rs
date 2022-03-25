#[allow(unused_imports)]
use core::arch::asm;
use tock_registers::interfaces::Readable;

pub use cortex_a::asm::*;

/// Returns the current stack pointer.
#[inline]
pub fn sp() -> *const u8 {
    #[cfg(target_arch = "aarch64")]
    unsafe {
        let ptr: usize;
        asm!("mov {0}, sp", out(reg) ptr, options(pure, nomem, nostack));
        ptr as *const u8
    }

    #[cfg(not(target_arch = "aarch64"))]
    unimplemented!()
}

/// # Safety
///
/// Returns the current point counter.
#[inline]
pub unsafe fn pc() -> usize {
    #[cfg(target_arch = "aarch64")]
    {
        let pc: usize;
        asm!("adr {}, .", out(reg) pc, options(pure, nomem, nostack));
        pc
    }

    #[cfg(not(target_arch = "aarch64"))]
    unimplemented!()
}

/// CPU id
#[inline]
pub fn cpuid() -> usize {
    (crate::registers::MPIDR_EL1.get() & 3) as usize
}

/// # Safety
///
/// The halt function stops the processor until the next interrupt arrives
#[inline]
pub fn halt() {
    wfi();
}
