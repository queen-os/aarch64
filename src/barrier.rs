// SPDX-License-Identifier: Apache-2.0 OR MIT
//
// Copyright (c) 2018-2020 by the author(s)
//
// Author(s):
//   - Andre Richter <andre.o.richter@gmail.com>

// Borrow implementations from the pending upstream ACLE implementation until it is merged.
// Afterwards, we'll probably just reexport them, hoping that the API doesn't change.
//
// https://github.com/rust-lang-nursery/stdsimd/pull/557

#[allow(clippy::missing_safety_doc)]
pub mod sealed {
    pub trait Dmb {
        unsafe fn __dmb(&self);
    }

    pub trait Dsb {
        unsafe fn __dsb(&self);
    }

    pub trait Isb {
        unsafe fn __isb(&self);
    }
}

macro_rules! dmb_dsb {
    ($A:ident) => {
        impl sealed::Dmb for $A {
            #[inline(always)]
            unsafe fn __dmb(&self) {
                asm!(concat!("DMB ", stringify!($A)), options(nostack))
            }
        }
        impl sealed::Dsb for $A {
            #[inline(always)]
            unsafe fn __dsb(&self) {
                asm!(concat!("DSB ", stringify!($A)), options(nostack))
            }
        }
    };
}
#[derive(Copy, Clone)]
pub struct SY;
#[derive(Copy, Clone)]
pub struct ISH;
#[derive(Copy, Clone)]
pub struct ISHST;

dmb_dsb!(ISH);
dmb_dsb!(ISHST);
dmb_dsb!(SY);

impl sealed::Isb for SY {
    #[inline(always)]
    unsafe fn __isb(&self) {
        asm!("ISB SY", options(nostack))
    }
}

/// # Safety
///
/// In your own hands, this is hardware land!
#[inline(always)]
pub unsafe fn dmb<A>(arg: A)
where
    A: sealed::Dmb,
{
    arg.__dmb()
}

/// # Safety
///
/// In your own hands, this is hardware land!
#[inline(always)]
pub unsafe fn dsb<A>(arg: A)
where
    A: sealed::Dsb,
{
    arg.__dsb()
}

/// # Safety
///
/// In your own hands, this is hardware land!
#[inline(always)]
pub unsafe fn isb<A>(arg: A)
where
    A: sealed::Isb,
{
    arg.__isb()
}
