use bit_field::BitField;
use bitflags::_core::fmt::Formatter;
use core::convert::{Into, TryInto};
use core::fmt;
use core::hash::Hash;
use core::ops;
use ux::{u12, u21, u9};

pub const PAGE_SIZE_4KIB: u64 = 0x0000_1000;
pub const PAGE_SIZE_2MIB: u64 = 0x0020_0000;
pub const PAGE_SIZE_1GIB: u64 = 0x4000_0000;

#[derive(Clone, Copy, Debug)]
#[repr(u8)]
pub enum VirtAddrRange {
    /// 0x0000000000000000 to 0x0000FFFFFFFFFFFF
    Bottom = 0,
    /// 0xFFFF000000000000 to 0xFFFFFFFFFFFFFFFF.
    Top = 1,
}

impl VirtAddrRange {
    /// Returns the address offset
    pub fn as_offset(&self) -> u64 {
        match self {
            VirtAddrRange::Bottom => 0,
            VirtAddrRange::Top => 0xFFFF_0000_0000_0000,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct VirtAddr(u64);

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct PhysAddr(u64);

impl VirtAddr {
    #[inline]
    pub fn new(addr: u64) -> Self {
        VirtAddr(addr)
    }

    /// Tries to create a new canonical virtual address.
    /// in aarch64, valid virtual address starts with 0x0000 or 0xffff.
    #[inline]
    pub fn try_new(addr: u64) -> Option<Self> {
        match addr.get_bits(48..64) {
            0 | 0xffff => Some(VirtAddr(addr)),
            _ => None,
        }
    }

    /// Creates a virtual address that points to `0`.
    pub const fn zero() -> Self {
        VirtAddr(0)
    }

    #[inline]
    pub fn as_u64(self) -> u64 {
        self.0
    }

    #[inline]
    pub fn is_zero(self) -> bool {
        self.0 == 0
    }

    /// Creates a virtual address from the given pointer
    #[inline]
    pub fn from_ptr<T>(ptr: *const T) -> Self {
        Self::new(cast::u64(ptr as usize))
    }

    /// Converts the address to a raw pointer.
    #[cfg(target_pointer_width = "64")]
    #[inline]
    pub fn as_ptr<T>(self) -> *const T {
        cast::usize(self.as_u64()) as *const T
    }

    /// Converts the address to a mutable raw pointer.
    #[cfg(target_pointer_width = "64")]
    #[inline]
    pub fn as_mut_ptr<T>(self) -> *mut T {
        self.as_ptr::<T>() as *mut T
    }

    /// Aligns the virtual address upwards to the given alignment.
    ///
    /// See the `align_up` function for more information.
    #[inline]
    pub fn align_up<U>(self, align: U) -> Self
        where
            U: Into<u64>,
    {
        VirtAddr(align_up(self.0, align.into()))
    }

    /// Aligns the virtual address downwards to the given alignment.
    ///
    /// See the `align_down` function for more information.
    #[inline]
    pub fn align_down<U>(self, align: U) -> Self
        where
            U: Into<u64>,
    {
        VirtAddr(align_down(self.0, align.into()))
    }

    /// Checks whether the virtual address has the demanded alignment.
    #[inline]
    pub fn is_aligned<U>(self, align: U) -> bool
        where
            U: Into<u64>,
    {
        self.align_down(align) == self
    }

    /// Offset within the 4 KiB page.
    #[inline]
    pub fn page_offset(self) -> u12 {
        u12::new((self.0 & (PAGE_SIZE_4KIB - 1)) as u16)
    }

    /// Offset within the 2 MiB page.
    #[inline]
    pub fn large_page_offset(self) -> u21 {
        u21::new((self.0 & (PAGE_SIZE_2MIB - 1)) as u32)
    }

    /// Returns the VA range
    pub fn va_range(self) -> Option<VirtAddrRange> {
        match self.va_range_bits() {
            0x0000 => Some(VirtAddrRange::Bottom),
            0xffff => Some(VirtAddrRange::Top),
            _ => None,
        }
    }

    /// Returns the top 16 bits
    #[inline]
    pub fn va_range_bits(self) -> u16 {
        self.0.get_bits(48..64) as u16
    }

    /// Returns the 9-bit level 1 page table index.
    #[inline]
    pub fn p1_index(self) -> u9 {
        u9::new(((self.0 >> 12) & 0o777).try_into().unwrap())
    }

    /// Returns the 9-bit level 2 page table index.
    #[inline]
    pub fn p2_index(self) -> u9 {
        u9::new(((self.0 >> 12 >> 9) & 0o777).try_into().unwrap())
    }

    /// Returns the 9-bit level 3 page table index.
    #[inline]
    pub fn p3_index(self) -> u9 {
        u9::new(((self.0 >> 12 >> 9 >> 9) & 0o777).try_into().unwrap())
    }

    /// Returns the 9-bit level 4 page table index.
    #[inline]
    pub fn p4_index(self) -> u9 {
        u9::new(((self.0 >> 12 >> 9 >> 9 >> 9) & 0o777).try_into().unwrap())
    }
}

impl From<u64> for VirtAddr {
    fn from(num: u64) -> Self {
        VirtAddr(num)
    }
}

impl Into<u64> for VirtAddr {
    fn into(self) -> u64 {
        self.0
    }
}

impl ops::Add for VirtAddr {
    type Output = VirtAddr;

    fn add(self, rhs: VirtAddr) -> Self::Output {
        VirtAddr(self.0 + rhs.0)
    }
}

impl ops::Add<u64> for VirtAddr {
    type Output = VirtAddr;

    fn add(self, rhs: u64) -> Self::Output {
        VirtAddr(self.0 + rhs)
    }
}

impl ops::Add<usize> for VirtAddr {
    type Output = Self;
    fn add(self, rhs: usize) -> Self::Output {
        self + cast::u64(rhs)
    }
}

impl ops::AddAssign for VirtAddr {
    fn add_assign(&mut self, other: VirtAddr) {
        *self = VirtAddr::from(self.0 + other.0);
    }
}

impl ops::AddAssign<u64> for VirtAddr {
    fn add_assign(&mut self, offset: u64) {
        *self = VirtAddr::from(self.0 + offset);
    }
}

impl ops::AddAssign<usize> for VirtAddr {
    fn add_assign(&mut self, rhs: usize) {
        self.add_assign(cast::u64(rhs))
    }
}

impl ops::Sub for VirtAddr {
    type Output = u64;

    fn sub(self, rhs: VirtAddr) -> Self::Output {
        self.0 - rhs.0
    }
}

impl ops::Sub<u64> for VirtAddr {
    type Output = VirtAddr;

    fn sub(self, rhs: u64) -> Self::Output {
        VirtAddr::from(self.0 - rhs)
    }
}

impl ops::Sub<usize> for VirtAddr {
    type Output = Self;
    fn sub(self, rhs: usize) -> Self::Output {
        self - cast::u64(rhs)
    }
}

impl ops::SubAssign<u64> for VirtAddr {
    fn sub_assign(&mut self, rhs: u64) {
        *self = *self - rhs;
    }
}

impl ops::SubAssign<usize> for VirtAddr {
    fn sub_assign(&mut self, rhs: usize) {
        self.sub_assign(cast::u64(rhs))
    }
}

impl ops::Rem for VirtAddr {
    type Output = VirtAddr;

    fn rem(self, rhs: VirtAddr) -> Self::Output {
        VirtAddr(self.0 % rhs.0)
    }
}

impl ops::Rem<u64> for VirtAddr {
    type Output = u64;

    fn rem(self, rhs: Self::Output) -> Self::Output {
        self.0 % rhs
    }
}

impl ops::BitAnd for VirtAddr {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        VirtAddr(self.0 & rhs.0)
    }
}

impl ops::BitAnd<u64> for VirtAddr {
    type Output = VirtAddr;

    fn bitand(self, rhs: u64) -> Self::Output {
        VirtAddr(self.0 & rhs)
    }
}

impl ops::BitOr for VirtAddr {
    type Output = VirtAddr;

    fn bitor(self, rhs: VirtAddr) -> VirtAddr {
        VirtAddr(self.0 | rhs.0)
    }
}

impl ops::BitOr<u64> for VirtAddr {
    type Output = VirtAddr;

    fn bitor(self, rhs: u64) -> Self::Output {
        VirtAddr(self.0 | rhs)
    }
}

impl ops::Shr<u64> for VirtAddr {
    type Output = u64;

    fn shr(self, rhs: u64) -> Self::Output {
        self.0 >> rhs as u64
    }
}

impl fmt::Binary for VirtAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl fmt::Display for VirtAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:#x}", self.0)
    }
}

impl fmt::Debug for VirtAddr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "VirtAddr({:#x})", self.0)
    }
}

impl fmt::LowerHex for VirtAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl fmt::Octal for VirtAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl fmt::UpperHex for VirtAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl fmt::Pointer for VirtAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use core::fmt::LowerHex;
        self.0.fmt(f)
    }
}

impl PhysAddr {
    #[inline]
    pub fn new(addr: u64) -> Self {
        PhysAddr(addr)
    }

    /// Tries to create a new physical address.
    ///
    /// Fails if any bits in the range 52 to 64 are set.
    #[inline]
    pub fn try_new(addr: u64) -> Option<Self> {
        match addr.get_bits(52..64) {
            0 => Some(PhysAddr(addr)), // valid
            _ => None,
        }
    }

    pub const fn zero() -> Self {
        PhysAddr(0)
    }

    #[inline]
    pub fn as_u64(self) -> u64 {
        self.0
    }

    #[inline]
    pub fn is_zero(self) -> bool {
        self.0 == 0
    }

    /// Aligns the physical address upwards to the given alignment.
    ///
    /// See the `align_up` function for more information.
    #[inline]
    pub fn align_up<U>(self, align: U) -> Self
        where
            U: Into<u64>,
    {
        PhysAddr(align_up(self.0, align.into()))
    }

    /// Aligns the physical address downwards to the given alignment.
    ///
    /// See the `align_down` function for more information.
    #[inline]
    pub fn align_down<U>(self, align: U) -> Self
        where
            U: Into<u64>,
    {
        PhysAddr(align_down(self.0, align.into()))
    }

    /// Checks whether the physical address has the demanded alignment.
    #[inline]
    pub fn is_aligned<U>(self, align: U) -> bool
        where
            U: Into<u64>,
    {
        self.align_down(align) == self
    }
}

impl From<u64> for PhysAddr {
    fn from(num: u64) -> Self {
        PhysAddr(num)
    }
}

impl Into<u64> for PhysAddr {
    fn into(self) -> u64 {
        self.0
    }
}

impl ops::Add for PhysAddr {
    type Output = PhysAddr;

    fn add(self, rhs: PhysAddr) -> Self::Output {
        PhysAddr(self.0 + rhs.0)
    }
}

impl ops::Add<u64> for PhysAddr {
    type Output = PhysAddr;

    fn add(self, rhs: u64) -> Self::Output {
        PhysAddr(self.0 + rhs)
    }
}

impl ops::Add<usize> for PhysAddr {
    type Output = Self;
    fn add(self, rhs: usize) -> Self::Output {
        self + cast::u64(rhs)
    }
}

impl ops::AddAssign for PhysAddr {
    fn add_assign(&mut self, other: PhysAddr) {
        *self = PhysAddr::from(self.0 + other.0);
    }
}

impl ops::AddAssign<u64> for PhysAddr {
    fn add_assign(&mut self, offset: u64) {
        *self = PhysAddr::from(self.0 + offset);
    }
}

impl ops::AddAssign<usize> for PhysAddr {
    fn add_assign(&mut self, rhs: usize) {
        self.add_assign(cast::u64(rhs))
    }
}

impl ops::Sub for PhysAddr {
    type Output = u64;

    fn sub(self, rhs: PhysAddr) -> Self::Output {
        self.0 - rhs.0
    }
}

impl ops::Sub<u64> for PhysAddr {
    type Output = PhysAddr;

    fn sub(self, rhs: u64) -> Self::Output {
        PhysAddr::from(self.0 - rhs)
    }
}

impl ops::Sub<usize> for PhysAddr {
    type Output = Self;
    fn sub(self, rhs: usize) -> Self::Output {
        self - cast::u64(rhs)
    }
}

impl ops::SubAssign<u64> for PhysAddr {
    fn sub_assign(&mut self, rhs: u64) {
        *self = *self - rhs;
    }
}

impl ops::SubAssign<usize> for PhysAddr {
    fn sub_assign(&mut self, rhs: usize) {
        self.sub_assign(cast::u64(rhs))
    }
}

impl ops::Rem for PhysAddr {
    type Output = PhysAddr;

    fn rem(self, rhs: PhysAddr) -> Self::Output {
        PhysAddr(self.0 % rhs.0)
    }
}

impl ops::Rem<u64> for PhysAddr {
    type Output = u64;

    fn rem(self, rhs: Self::Output) -> Self::Output {
        self.0 % rhs
    }
}

impl ops::BitAnd for PhysAddr {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        PhysAddr(self.0 & rhs.0)
    }
}

impl ops::BitAnd<u64> for PhysAddr {
    type Output = PhysAddr;

    fn bitand(self, rhs: u64) -> Self::Output {
        PhysAddr(self.0 & rhs)
    }
}

impl ops::BitOr for PhysAddr {
    type Output = PhysAddr;

    fn bitor(self, rhs: PhysAddr) -> PhysAddr {
        PhysAddr(self.0 | rhs.0)
    }
}

impl ops::BitOr<u64> for PhysAddr {
    type Output = PhysAddr;

    fn bitor(self, rhs: u64) -> Self::Output {
        PhysAddr(self.0 | rhs)
    }
}

impl ops::Shr<u64> for PhysAddr {
    type Output = u64;

    fn shr(self, rhs: u64) -> Self::Output {
        self.0 >> rhs as u64
    }
}

impl fmt::Binary for PhysAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl fmt::Display for PhysAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:#x}", self.0)
    }
}

impl fmt::Debug for PhysAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "PhysAddr({:#x})", self.0)
    }
}

impl fmt::LowerHex for PhysAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl fmt::Octal for PhysAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl fmt::UpperHex for PhysAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl fmt::Pointer for PhysAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use core::fmt::LowerHex;
        self.0.fmt(f)
    }
}

/// Align address downwards.
///
/// Returns the greatest x with alignment `align` so that x <= addr. The alignment must be
///  a power of 2.
#[inline]
pub fn align_down(addr: u64, align: u64) -> u64 {
    debug_assert!(align.is_power_of_two(), "`align` must be a power of two");
    addr & !(align - 1)
}

/// Align address upwards.
///
/// Returns the smallest x with alignment `align` so that x >= addr. The alignment must be
/// a power of 2.
#[inline]
pub fn align_up(addr: u64, align: u64) -> u64 {
    debug_assert!(align.is_power_of_two(), "`align` must be a power of two");
    let align_mask = align - 1;
    if addr & align_mask == 0 {
        addr // already aligned
    } else {
        (addr | align_mask) + 1
    }
}
