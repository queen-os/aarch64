use crate::{
    addr::{PhysAddr, VirtAddr},
    barrier,
    paging::Frame,
    registers::*,
};
use core::arch::asm;

/// Address Translate (Stage 1 EL1 Read).
///
/// For Raspi 3, it always return the result of a translation table walk,
/// regardless of the TLB caching.
#[inline]
pub fn address_translate(vaddr: usize) -> usize {
    unsafe {
        asm!("at S1E1R, {}", in(reg) vaddr);
        barrier::isb(barrier::SY);
    }
    PAR_EL1.get() as usize
}

/// Read TTBRx_EL1 as Frame
#[inline]
pub fn ttbr_el1_read(which: u8) -> Frame {
    let baddr = match which {
        0 => TTBR0_EL1.get_baddr(),
        1 => TTBR1_EL1.get_baddr(),
        _ => 0,
    };
    Frame::containing_address(PhysAddr::new(baddr))
}

/// Write TTBRx_EL1 from Frame
#[inline]
pub fn ttbr_el1_write(which: u8, frame: Frame) {
    let baddr = frame.start_address().as_u64();
    match which {
        0 => TTBR0_EL1.set_baddr(baddr),
        1 => TTBR1_EL1.set_baddr(baddr),
        _ => {}
    };
}

/// Read TTBRx_EL1 as Frame and ASID
#[inline]
pub fn ttbr_el1_read_asid(which: u8) -> (u16, Frame) {
    let (asid, baddr) = match which {
        0 => (TTBR0_EL1.get_asid(), TTBR0_EL1.get_baddr()),
        1 => (TTBR1_EL1.get_asid(), TTBR1_EL1.get_baddr()),
        _ => (0, 0),
    };
    (asid, Frame::containing_address(PhysAddr::new(baddr)))
}

/// write TTBRx_EL1 from Frame and ASID
#[inline]
pub fn ttbr_el1_write_asid(which: u8, asid: u16, frame: Frame) {
    let baddr = frame.start_address().as_u64();
    match which {
        0 => TTBR0_EL1.write(TTBR0_EL1::ASID.val(asid as u64) + TTBR0_EL1::BADDR.val(baddr >> 1)),
        1 => TTBR1_EL1.write(TTBR1_EL1::ASID.val(asid as u64) + TTBR1_EL1::BADDR.val(baddr >> 1)),
        _ => {}
    };
}

/// Invalidate all TLB entries in all PEs.
#[inline]
pub fn invalidate_tlb_all() {
    // All stage 1 translations used at EL1, in the Inner Shareable shareability
    // domain.
    unsafe {
        asm!(
            "dsb ishst
             tlbi vmalle1is
             dsb ish
             isb"
        );
    }
}

/// Invalidate all TLB entries in the current PE.
#[inline]
pub fn local_invalidate_tlb_all() {
    // All stage 1 translations used at EL1
    unsafe {
        asm!(
            "dsb nshst
             tlbi vmalle1
             dsb nsh
             isb"
        );
    }
}

/// Invalidate TLB entries in all PEs by the virtual address.
#[inline]
pub fn invalidate_tlb_vaddr(vaddr: VirtAddr) {
    // Translations used at EL1 for the specified address, for all ASID values,
    // in the Inner Shareable shareability domain.
    unsafe {
        asm!(
            "dsb ishst
             tlbi vaae1is, {}
             dsb ish
             isb",
            in(reg) vaddr.as_u64() >> 12
        );
    }
}
