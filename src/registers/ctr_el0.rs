//! Cache Type Register
//!
//! Provides information about the architecture of the caches.

use tock_registers::{
    interfaces::{Readable, Writeable},
    register_bitfields,
};

register_bitfields! {u64,
    pub CTR_EL0 [
        /// When FEAT_MTE is implemented:
        /// Tag minimum Line. Log2 of the number of words covered by Allocation Tags in the smallest cache
        /// line of all caches which can contain Allocation tags that are controlled by the PE.
        TminLine OFFSET(32) NUMBITS(6) [],

        /// Instruction cache invalidation requirements for data to instruction coherence
        DIC OFFSET(29) NUMBITS(1) [
            REQUIRED = 0,
            NOT_REQUIRED = 1
        ],

        /// Data cache clean requirements for instruction to data coherence.
        IDC OFFSET(28) NUMBITS(1) [
            /// Data cache clean to the Point of Unification is required for instruction to data coherence,
            /// unless CLIDR_EL1.LoC == 0b000 or (CLIDR_EL1.LoUIS == 0b000 && CLIDR_EL1.LoUU == 0b000).
            REQUIRED = 0,
            NOT_REQUIRED = 1
        ],

        /// Cache writeback granule. Log_2_ of the number of words of the maximum size of memory that can be
        /// overwritten as a result of the eviction of a cache entry that has had a memory location in it modified.
        /// A value of 0b0000 indicates that this register does not provide Cache writeback granule information and either:
        /// • The architectural maximum of 512 words (2KB) must be assumed.
        /// • The Cache writeback granule can be determined from maximum cache line size encoded in the Cache Size ID Registers.
        /// Values greater than 0b1001 are reserved.
        /// Arm recommends that an implementation that does not support cache write-back implements this
        /// field as 0b0001. This applies, for example, to an implementation that supports only write-through caches.
        CWG OFFSET(24) NUMBITS(4) [],

        /// Log2 of the number of words in the smallest cache line of all the
        /// data caches and unified caches that are controlled by the PE.
        DminLine OFFSET(16) NUMBITS(4) [],

        /// Level 1 instruction cache policy. Indicates the indexing and tagging
        /// policy for the L1 instruction cache. Possible values of this field are:
        /// The value 0b01 is reserved in ARMv8.
        /// The value 0b00 is permitted only in an implementation that includes
        /// ARMv8.2-VPIPT, otherwise the value is reserved.
        L1Ip OFFSET(14) NUMBITS(2) [
            /// VMID aware Physical Index, Physical tag (VPIPT)
            VPIPT = 0b00,
            /// ASID-tagged Virtual Index, Virtual Tag (AIVIVT)
            AIVIVT = 0b01,
            /// Virtual Index, Physical Tag (VIPT)
            VIPT = 0b10,
            /// Physical Index, Physical Tag (PIPT)
            PIPT = 0b11
        ],

        /// Log2 of the number of words in the smallest cache line of all the
        /// instruction caches that are controlled by the PE.
        IminLine OFFSET(0) NUMBITS(4) []
    ]
}

pub struct Reg;

impl Readable for Reg {
    type T = u64;
    type R = CTR_EL0::Register;

    sys_coproc_read_raw!(u64, "CTR_EL0", "x");
}

impl Writeable for Reg {
    type T = u64;
    type R = CTR_EL0::Register;

    sys_coproc_write_raw!(u64, "CTR_EL0", "x");
}

pub static CTR_EL0: Reg = Reg {};
