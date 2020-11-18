use super::{NotGiantPageSize, PageSize, Size1GiB, Size2MiB, Size4KiB};
use crate::addr::{VirtAddr, VirtAddrRange};
use core::convert::TryFrom;
use core::fmt;
use core::marker::PhantomData;
use core::ops::{Add, AddAssign, Range, Sub, SubAssign};
use ux::u9;

/// A virtual memory page.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(C)]
pub struct Page<S: PageSize = Size4KiB> {
    start_address: VirtAddr,
    size: PhantomData<S>,
}

impl<S: PageSize> Page<S> {
    /// The page size in bytes.
    pub const SIZE: u64 = S::SIZE;

    /// Returns the page that starts at the given virtual address.
    ///
    /// Returns `None` if the address is not correctly aligned (i.e. is not a valid page start).
    pub fn from_start_address(address: VirtAddr) -> Option<Self> {
        address
            .is_aligned(S::SIZE)
            .then_some(Page::containing_address(address))
    }

    /// Returns the page that contains the given virtual address.
    pub fn containing_address(address: VirtAddr) -> Self {
        Page {
            start_address: address.align_down(S::SIZE),
            size: PhantomData,
        }
    }

    /// Returns the start address of the page.
    pub fn start_address(&self) -> VirtAddr {
        self.start_address
    }

    /// Returns the size the page (4KB, 2MB or 1GB).
    pub const fn size(&self) -> u64 {
        S::SIZE
    }

    /// Returns the VA range
    pub fn va_range(&self) -> Option<VirtAddrRange> {
        self.start_address().va_range()
    }

    /// Returns the level 4 page table index of this page.
    pub fn p4_index(&self) -> u9 {
        self.start_address().p4_index()
    }

    /// Returns the level 3 page table index of this page.
    pub fn p3_index(&self) -> u9 {
        self.start_address().p3_index()
    }

    pub fn of_addr(address: u64) -> Self {
        Self::containing_address(VirtAddr::new(address))
    }

    pub fn range_of(begin: u64, end: u64) -> Range<Self> {
        Page::of_addr(begin)..(Page::of_addr(end - 1) + 1)
    }
}

impl<S: NotGiantPageSize> Page<S> {
    /// Returns the level 2 page table index of this page.
    pub fn p2_index(&self) -> u9 {
        self.start_address().p2_index()
    }
}

impl Page<Size1GiB> {
    /// Returns the 1GiB memory page with the specified page table indices.
    pub fn from_page_table_indices_1gib(
        va_range: VirtAddrRange,
        p4_index: u9,
        p3_index: u9,
    ) -> Self {
        use bit_field::BitField;

        let mut addr = va_range.as_offset();
        addr.set_bits(39..48, u64::from(p4_index));
        addr.set_bits(30..39, u64::from(p3_index));
        Page::containing_address(VirtAddr::new(addr))
    }
}

impl Page<Size2MiB> {
    /// Returns the 2MiB memory page with the specified page table indices.
    pub fn from_page_table_indices_2mib(
        va_range: VirtAddrRange,
        p4_index: u9,
        p3_index: u9,
        p2_index: u9,
    ) -> Self {
        use bit_field::BitField;

        let mut addr = va_range.as_offset();
        addr.set_bits(39..48, u64::from(p4_index));
        addr.set_bits(30..39, u64::from(p3_index));
        addr.set_bits(21..30, u64::from(p2_index));
        Page::containing_address(VirtAddr::new(addr))
    }
}

impl Page<Size4KiB> {
    /// Returns the 4KiB memory page with the specified page table indices.
    pub fn from_page_table_indices(
        va_range: VirtAddrRange,
        p4_index: u9,
        p3_index: u9,
        p2_index: u9,
        p1_index: u9,
    ) -> Self {
        use bit_field::BitField;

        let mut addr = va_range.as_offset();
        addr.set_bits(39..48, u64::from(p4_index));
        addr.set_bits(30..39, u64::from(p3_index));
        addr.set_bits(21..30, u64::from(p2_index));
        addr.set_bits(12..21, u64::from(p1_index));
        Page::containing_address(VirtAddr::new(addr))
    }

    /// Returns the level 1 page table index of this page.
    pub fn p1_index(&self) -> u9 {
        self.start_address().p1_index()
    }
}

impl<S: PageSize> fmt::Debug for Page<S> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_fmt(format_args!(
            "Page[{}]({:#x})",
            S::SIZE_AS_DEBUG_STR,
            self.start_address().as_u64()
        ))
    }
}

unsafe impl<S: PageSize> core::iter::Step for Page<S> {
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

impl<S: PageSize> Add<u64> for Page<S> {
    type Output = Self;
    fn add(self, rhs: u64) -> Self::Output {
        Page::containing_address(self.start_address() + rhs * S::SIZE)
    }
}

impl<S: PageSize> AddAssign<u64> for Page<S> {
    fn add_assign(&mut self, rhs: u64) {
        *self = *self + rhs;
    }
}

impl<S: PageSize> Sub<u64> for Page<S> {
    type Output = Self;
    fn sub(self, rhs: u64) -> Self::Output {
        Page::containing_address(self.start_address() - rhs * S::SIZE)
    }
}

impl<S: PageSize> SubAssign<u64> for Page<S> {
    fn sub_assign(&mut self, rhs: u64) {
        *self = *self - rhs;
    }
}

impl<S: PageSize> Sub<Self> for Page<S> {
    type Output = u64;
    fn sub(self, rhs: Self) -> Self::Output {
        (self.start_address - rhs.start_address) / S::SIZE
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_page_ranges() {
        let page_size = Size4KiB::SIZE;
        let number = 1000;

        let start_addr = VirtAddr::new(0xdeadbeaf);
        let start: Page = Page::containing_address(start_addr);
        let end = start + number;
        let mut range = start..end;

        for i in 0..number {
            assert_eq!(
                range.next(),
                Some(Page::containing_address(start_addr + page_size * i))
            );
        }
        assert_eq!(range.next(), None);

        let mut range_inclusive = start..=end;
        for i in 0..=number {
            assert_eq!(
                range_inclusive.next(),
                Some(Page::containing_address(start_addr + page_size * i))
            );
        }
        assert_eq!(range_inclusive.next(), None);
    }
}
