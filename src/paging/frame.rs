use super::{PageSize, Size4KiB};
use crate::addr::PhysAddr;
use core::{
    convert::TryFrom,
    fmt,
    marker::PhantomData,
    ops::{Add, AddAssign, Range, Sub, SubAssign},
};

/// A physical memory frame.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(C)]
pub struct Frame<S: PageSize = Size4KiB> {
    start_address: PhysAddr,
    size: PhantomData<S>,
}

impl<S: PageSize> Frame<S> {
    /// Returns the frame that starts at the given virtual address.
    ///
    /// Returns `None` if the address is not correctly aligned (i.e. is not a valid frame start).
    #[inline]
    pub fn from_start_address(address: PhysAddr) -> Option<Self> {
        address
            .is_aligned(S::SIZE)
            .then_some(Frame::containing_address(address))
    }

    /// Returns the frame that contains the given physical address.
    #[inline]
    pub fn containing_address(address: PhysAddr) -> Self {
        Frame {
            start_address: address.align_down(S::SIZE),
            size: PhantomData,
        }
    }

    /// Returns the start address of the frame.
    #[inline]
    pub fn start_address(self) -> PhysAddr {
        self.start_address
    }

    /// Returns the size the frame (4KB, 2MB or 1GB).
    pub const fn size(&self) -> u64 {
        S::SIZE
    }

    #[inline]
    pub fn of_addr(address: u64) -> Self {
        Self::containing_address(PhysAddr::new(address))
    }

    #[inline]
    pub fn range_of(begin: u64, end: u64) -> Range<Self> {
        Frame::of_addr(begin)..Frame::of_addr(end)
    }
}

impl<S: PageSize> fmt::Debug for Frame<S> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_fmt(format_args!(
            "PhysFrame[{}]({:#x})",
            S::SIZE_AS_DEBUG_STR,
            self.start_address().as_u64()
        ))
    }
}

impl<S: PageSize> Add<u64> for Frame<S> {
    type Output = Self;
    fn add(self, rhs: u64) -> Self::Output {
        Frame::containing_address(self.start_address() + rhs * S::SIZE)
    }
}

impl<S: PageSize> AddAssign<u64> for Frame<S> {
    fn add_assign(&mut self, rhs: u64) {
        *self = *self + rhs;
    }
}

impl<S: PageSize> Sub<u64> for Frame<S> {
    type Output = Self;
    fn sub(self, rhs: u64) -> Self::Output {
        Frame::containing_address(self.start_address() - rhs * S::SIZE)
    }
}

impl<S: PageSize> SubAssign<u64> for Frame<S> {
    fn sub_assign(&mut self, rhs: u64) {
        *self = *self - rhs;
    }
}

impl<S: PageSize> Sub<Frame<S>> for Frame<S> {
    type Output = u64;
    fn sub(self, rhs: Frame<S>) -> Self::Output {
        (self.start_address - rhs.start_address) / S::SIZE
    }
}

impl<S: PageSize> core::iter::Step for Frame<S> {
    fn steps_between(start: &Self, end: &Self) -> Option<usize> {
        usize::try_from(*end - *start).ok()
    }

    fn forward_checked(start: Self, count: usize) -> Option<Self> {
        let page = start + count as u64;
        (page.start_address >= start.start_address).then_some(page)
    }

    fn backward_checked(start: Self, count: usize) -> Option<Self> {
        let page = start - count as u64;
        (page.start_address <= start.start_address).then_some(page)
    }
}
