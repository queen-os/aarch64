//! Memory region attributes (D4.5, page 2174)

use crate::paging::table::{PageTableAttribute, MEMORY_ATTRIBUTE};

tock_registers::register_bitfields! {u64,
    pub MAIR_ATTR [
        Attr_HIGH OFFSET(4) NUMBITS(4) [
            Device = 0b0000,
            Memory_OuterNonCacheable = 0b0100,
            Memory_OuterWriteThrough_NonTransient_ReadAlloc_WriteAlloc = 0b1011,
            Memory_OuterWriteBack_NonTransient_ReadAlloc_WriteAlloc = 0b1111
        ],
        Attr_LOW_DEVICE OFFSET(0) NUMBITS(4) [
            Device_nGnRnE = 0b0000,
            Device_nGnRE  = 0b0100,
            Device_nGRE   = 0b1000,
            Device_GRE    = 0b1100
        ],
        Attr_LOW_MEMORY OFFSET(0) NUMBITS(4) [
            InnerNonCacheable = 0b0100,
            InnerWriteThrough_NonTransient_ReadAlloc_WriteAlloc = 0b1011,
            InnerWriteBack_NonTransient_ReadAlloc_WriteAlloc = 0b1111
        ]
    ]
}

pub trait MairType {
    const INDEX: u64;

    fn config_value() -> u64;

    fn attr_value() -> PageTableAttribute;
}

pub enum MairDevice {}
pub enum MairNormal {}
pub enum MairNormalNonCacheable {}

impl MairType for MairNormal {
    const INDEX: u64 = 0;

    #[inline]
    fn config_value() -> u64 {
        (MAIR_ATTR::Attr_HIGH::Memory_OuterWriteBack_NonTransient_ReadAlloc_WriteAlloc
            + MAIR_ATTR::Attr_LOW_MEMORY::InnerWriteBack_NonTransient_ReadAlloc_WriteAlloc)
            .value
    }

    #[inline]
    fn attr_value() -> PageTableAttribute {
        MEMORY_ATTRIBUTE::SH::InnerShareable + MEMORY_ATTRIBUTE::AttrIndx.val(Self::INDEX)
    }
}

impl MairType for MairDevice {
    const INDEX: u64 = 1;

    #[inline]
    fn config_value() -> u64 {
        (MAIR_ATTR::Attr_HIGH::Device + MAIR_ATTR::Attr_LOW_DEVICE::Device_nGnRE).value
    }

    #[inline]
    fn attr_value() -> PageTableAttribute {
        MEMORY_ATTRIBUTE::SH::OuterShareable + MEMORY_ATTRIBUTE::AttrIndx.val(Self::INDEX)
    }
}

impl MairType for MairNormalNonCacheable {
    const INDEX: u64 = 2;

    #[inline]
    fn config_value() -> u64 {
        (MAIR_ATTR::Attr_HIGH::Memory_OuterNonCacheable
            + MAIR_ATTR::Attr_LOW_MEMORY::InnerNonCacheable)
            .value
    }

    #[inline]
    fn attr_value() -> PageTableAttribute {
        MEMORY_ATTRIBUTE::SH::OuterShareable + MEMORY_ATTRIBUTE::AttrIndx.val(Self::INDEX)
    }
}
