use crate::paging::{mapper::*, table::PageTable, Frame, Page, PageTableFlags};

/// A Mapper implementation that requires that the complete physically memory is mapped at some
/// offset in the virtual address space.
#[derive(Debug)]
pub struct OffsetPageTable<'a> {
    inner: MappedPageTable<'a, PhysOffset>,
}

impl<'a> OffsetPageTable<'a> {
    /// Creates a new `OffsetPageTable` that uses the given offset for converting virtual
    /// to physical addresses.
    ///
    /// The complete physical memory must be mapped in the virtual address space starting at
    /// address `phys_offset`. This means that for example physical address `0x5000` can be
    /// accessed through virtual address `phys_offset + 0x5000`. This mapping is required because
    /// the mapper needs to access page tables, which are not mapped into the virtual address
    /// space by default.
    ///
    /// ## Safety
    ///
    /// This function is unsafe because the caller must guarantee that the passed `phys_offset`
    /// is correct. Also, the passed `level_4_table` must point to the level 4 page table
    /// of a valid page table hierarchy. Otherwise this function might break memory safety, e.g.
    /// by writing to an illegal memory location.
    #[inline]
    pub unsafe fn new(level_4_table: &'a mut PageTable, phys_offset: VirtAddr) -> Self {
        let phys_offset = PhysOffset {
            offset: phys_offset,
        };
        Self {
            inner: MappedPageTable::new(level_4_table, phys_offset),
        }
    }

    /// Returns a mutable reference to the wrapped level 4 `PageTable` instance.
    pub fn level_4_table(&mut self) -> &mut PageTable {
        self.inner.level_4_table()
    }
}

#[derive(Debug)]
struct PhysOffset {
    offset: VirtAddr,
}

impl PhysToVirt for PhysOffset {
    #[inline]
    fn phys_to_virt(&self, frame: Frame) -> *mut PageTable {
        let phys = frame.start_address().as_u64();
        let virt = self.offset + phys;
        virt.as_mut_ptr()
    }
}

// delegate all trait implementations to inner

impl<'a> Mapper<Size1GiB> for OffsetPageTable<'a> {
    #[inline]
    unsafe fn map_to<A>(
        &mut self,
        page: Page<Size1GiB>,
        frame: Frame<Size1GiB>,
        flags: PageTableFlags,
        attr: PageTableAttribute,
        frame_allocator: &mut A,
    ) -> Result<MapperFlush<Size1GiB>, MapToError>
    where
        A: FrameAllocator<Size4KiB>,
    {
        self.inner.map_to(page, frame, flags, attr, frame_allocator)
    }

    #[inline]
    fn unmap(
        &mut self,
        page: Page<Size1GiB>,
    ) -> Result<(Frame<Size1GiB>, MapperFlush<Size1GiB>), UnmapError> {
        self.inner.unmap(page)
    }

    #[inline]
    fn translate_page(&self, page: Page<Size1GiB>) -> Result<Frame<Size1GiB>, TranslateError> {
        self.inner.translate_page(page)
    }

    #[inline]
    fn get_entry(&self, page: Page<Size1GiB>) -> Result<&PageTableEntry, EntryGetError> {
        self.inner.get_entry(page)
    }

    #[inline]
    fn get_entry_mut(
        &mut self,
        page: Page<Size1GiB>,
    ) -> Result<&mut PageTableEntry, EntryGetError> {
        self.inner.get_entry_mut(page)
    }
}

impl<'a> Mapper<Size2MiB> for OffsetPageTable<'a> {
    #[inline]
    unsafe fn map_to<A>(
        &mut self,
        page: Page<Size2MiB>,
        frame: Frame<Size2MiB>,
        flags: PageTableFlags,
        attr: PageTableAttribute,
        frame_allocator: &mut A,
    ) -> Result<MapperFlush<Size2MiB>, MapToError>
    where
        A: FrameAllocator<Size4KiB>,
    {
        self.inner.map_to(page, frame, flags, attr, frame_allocator)
    }

    #[inline]
    fn unmap(
        &mut self,
        page: Page<Size2MiB>,
    ) -> Result<(Frame<Size2MiB>, MapperFlush<Size2MiB>), UnmapError> {
        self.inner.unmap(page)
    }

    #[inline]
    fn translate_page(&self, page: Page<Size2MiB>) -> Result<Frame<Size2MiB>, TranslateError> {
        self.inner.translate_page(page)
    }

    #[inline]
    fn get_entry(&self, page: Page<Size2MiB>) -> Result<&PageTableEntry, EntryGetError> {
        self.inner.get_entry(page)
    }

    #[inline]
    fn get_entry_mut(
        &mut self,
        page: Page<Size2MiB>,
    ) -> Result<&mut PageTableEntry, EntryGetError> {
        self.inner.get_entry_mut(page)
    }
}

impl<'a> Mapper<Size4KiB> for OffsetPageTable<'a> {
    #[inline]
    unsafe fn map_to<A>(
        &mut self,
        page: Page<Size4KiB>,
        frame: Frame<Size4KiB>,
        flags: PageTableFlags,
        attr: PageTableAttribute,
        frame_allocator: &mut A,
    ) -> Result<MapperFlush<Size4KiB>, MapToError>
    where
        A: FrameAllocator<Size4KiB>,
    {
        self.inner.map_to(page, frame, flags, attr, frame_allocator)
    }

    #[inline]
    fn unmap(
        &mut self,
        page: Page<Size4KiB>,
    ) -> Result<(Frame<Size4KiB>, MapperFlush<Size4KiB>), UnmapError> {
        self.inner.unmap(page)
    }

    #[inline]
    fn translate_page(&self, page: Page<Size4KiB>) -> Result<Frame<Size4KiB>, TranslateError> {
        self.inner.translate_page(page)
    }

    #[inline]
    fn get_entry(&self, page: Page<Size4KiB>) -> Result<&PageTableEntry, EntryGetError> {
        self.inner.get_entry(page)
    }

    #[inline]
    fn get_entry_mut(
        &mut self,
        page: Page<Size4KiB>,
    ) -> Result<&mut PageTableEntry, EntryGetError> {
        self.inner.get_entry_mut(page)
    }
}

impl<'a> MapperAllSizes for OffsetPageTable<'a> {
    #[inline]
    fn translate(&self, addr: VirtAddr) -> TranslateResult {
        self.inner.translate(addr)
    }
}
