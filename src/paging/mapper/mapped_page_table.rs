//! Access the page tables through a normal level 4 table.

use crate::paging::{
    frame_alloc::FrameAllocator,
    mapper::*,
    table::{FrameError, PageTable, PageTableAttribute, PageTableEntry, PageTableFlags},
    Frame, Page, Size1GiB, Size2MiB, Size4KiB,
};

/// A Mapper implementation that relies on a PhysAddr to VirtAddr conversion function.
///
/// This type requires that the all physical page table frames are mapped to some virtual
/// address. Normally, this is done by mapping the complete physical address space into
/// the virtual address space at some offset. Other mappings between physical and virtual
/// memory are possible too, as long as they can be calculated as an `PhysAddr` to
/// `VirtAddr` closure.
#[derive(Debug)]
pub struct MappedPageTable<'a, P: PhysToVirt> {
    page_table_walker: PageTableWalker<P>,
    level_4_table: &'a mut PageTable,
}

impl<'a, P: PhysToVirt> MappedPageTable<'a, P> {
    /// Creates a new `MappedPageTable` that uses the passed closure for converting virtual
    /// to physical addresses.
    ///
    /// ## Safety
    ///
    /// This function is unsafe because the caller must guarantee that the passed `phys_to_virt`
    /// closure is correct. Also, the passed `level_4_table` must point to the level 4 page table
    /// of a valid page table hierarchy. Otherwise this function might break memory safety, e.g.
    /// by writing to an illegal memory location.
    #[inline]
    pub unsafe fn new(level_4_table: &'a mut PageTable, phys_to_virt: P) -> Self {
        Self {
            page_table_walker: PageTableWalker::new(phys_to_virt),
            level_4_table,
        }
    }

    /// Returns a mutable reference to the wrapped level 4 `PageTable` instance.
    #[inline]
    pub fn level_4_table(&mut self) -> &mut PageTable {
        &mut self.level_4_table
    }

    /// Helper function for implementing Mapper.
    fn map_to_1gib<A>(
        &mut self,
        page: Page<Size1GiB>,
        frame: Frame<Size1GiB>,
        flags: PageTableFlags,
        attr: PageTableAttribute,
        allocator: &mut A,
    ) -> Result<MapperFlush<Size1GiB>, MapToError>
    where
        A: FrameAllocator<Size4KiB>,
    {
        let p4 = &mut self.level_4_table;
        let p3 = self
            .page_table_walker
            .create_next_table(&mut p4[page.p4_index()], allocator)?;

        if !p3[page.p3_index()].is_unused() {
            return Err(MapToError::PageAlreadyMapped);
        }
        p3[page.p3_index()].set_block::<Size1GiB>(frame.start_address(), flags, attr);

        Ok(MapperFlush::new(page))
    }

    /// Helper function for implementing Mapper.
    fn map_to_2mib<A>(
        &mut self,
        page: Page<Size2MiB>,
        frame: Frame<Size2MiB>,
        flags: PageTableFlags,
        attr: PageTableAttribute,
        allocator: &mut A,
    ) -> Result<MapperFlush<Size2MiB>, MapToError>
    where
        A: FrameAllocator<Size4KiB>,
    {
        let p4 = &mut self.level_4_table;
        let p3 = self
            .page_table_walker
            .create_next_table(&mut p4[page.p4_index()], allocator)?;
        let p2 = self
            .page_table_walker
            .create_next_table(&mut p3[page.p3_index()], allocator)?;

        if !p2[page.p2_index()].is_unused() {
            return Err(MapToError::PageAlreadyMapped);
        }
        p2[page.p2_index()].set_block::<Size2MiB>(frame.start_address(), flags, attr);

        Ok(MapperFlush::new(page))
    }

    /// Helper function for implementing Mapper.
    fn map_to_4kib<A>(
        &mut self,
        page: Page<Size4KiB>,
        frame: Frame<Size4KiB>,
        flags: PageTableFlags,
        attr: PageTableAttribute,
        allocator: &mut A,
    ) -> Result<MapperFlush<Size4KiB>, MapToError>
    where
        A: FrameAllocator<Size4KiB>,
    {
        let p4 = &mut self.level_4_table;
        let p3 = self
            .page_table_walker
            .create_next_table(&mut p4[page.p4_index()], allocator)?;
        let p2 = self
            .page_table_walker
            .create_next_table(&mut p3[page.p3_index()], allocator)?;
        let p1 = self
            .page_table_walker
            .create_next_table(&mut p2[page.p2_index()], allocator)?;

        if !p1[page.p1_index()].is_unused() {
            return Err(MapToError::PageAlreadyMapped);
        }
        p1[page.p1_index()].set_frame(frame, flags, attr);

        Ok(MapperFlush::new(page))
    }
}

impl<'a, P: PhysToVirt> Mapper<Size1GiB> for MappedPageTable<'a, P> {
    #[inline]
    unsafe fn map_to<A>(
        &mut self,
        page: Page<Size1GiB>,
        frame: Frame<Size1GiB>,
        flags: PageTableFlags,
        attr: PageTableAttribute,
        allocator: &mut A,
    ) -> Result<MapperFlush<Size1GiB>, MapToError>
    where
        A: FrameAllocator<Size4KiB>,
    {
        self.map_to_1gib(page, frame, flags, attr, allocator)
    }

    fn unmap(
        &mut self,
        page: Page<Size1GiB>,
    ) -> Result<(Frame<Size1GiB>, MapperFlush<Size1GiB>), UnmapError> {
        let entry = self.get_entry_mut(page)?;

        if !entry.flags().contains(PageTableFlags::VALID) {
            return Err(UnmapError::PageNotMapped);
        } else if !entry.is_block() {
            return Err(UnmapError::ParentEntryHugePage);
        }

        let frame = Frame::from_start_address(entry.addr())
            .ok_or_else(|| UnmapError::InvalidFrameAddress(entry.addr()))?;

        entry.set_unused();
        Ok((frame, MapperFlush::new(page)))
    }

    fn get_entry(&self, page: Page<Size1GiB>) -> Result<&PageTableEntry, EntryGetError> {
        let p4 = &self.level_4_table;
        let p3 = self.page_table_walker.next_table(&p4[page.p4_index()])?;
        Ok(&p3[page.p3_index()])
    }

    fn get_entry_mut(&mut self, page: Page<Size1GiB>) -> Result<&mut PageTableEntry, EntryGetError> {
        let p4 = &mut self.level_4_table;
        let p3 = self.page_table_walker.next_table_mut(&mut p4[page.p4_index()])?;
        Ok(&mut p3[page.p3_index()])
    }
}

impl<'a, P: PhysToVirt> Mapper<Size2MiB> for MappedPageTable<'a, P> {
    #[inline]
    unsafe fn map_to<A>(
        &mut self,
        page: Page<Size2MiB>,
        frame: Frame<Size2MiB>,
        flags: PageTableFlags,
        attr: PageTableAttribute,
        allocator: &mut A,
    ) -> Result<MapperFlush<Size2MiB>, MapToError>
    where
        A: FrameAllocator<Size4KiB>,
    {
        self.map_to_2mib(page, frame, flags, attr, allocator)
    }

    fn unmap(
        &mut self,
        page: Page<Size2MiB>,
    ) -> Result<(Frame<Size2MiB>, MapperFlush<Size2MiB>), UnmapError> {
        let entry = self.get_entry_mut(page)?;

        if !entry.flags().contains(PageTableFlags::VALID) {
            return Err(UnmapError::PageNotMapped);
        } else if !entry.is_block() {
            return Err(UnmapError::ParentEntryHugePage);
        }

        let frame = Frame::from_start_address(entry.addr())
            .ok_or_else(|| UnmapError::InvalidFrameAddress(entry.addr()))?;

        entry.set_unused();
        Ok((frame, MapperFlush::new(page)))
    }

    fn get_entry(&self, page: Page<Size2MiB>) -> Result<&PageTableEntry, EntryGetError> {
        let p4 = &self.level_4_table;
        let p3 = self.page_table_walker.next_table(&p4[page.p4_index()])?;
        let p2 = self.page_table_walker.next_table(&p3[page.p3_index()])?;
        Ok(&p2[page.p2_index()])
    }

    fn get_entry_mut(&mut self, page: Page<Size2MiB>) -> Result<&mut PageTableEntry, EntryGetError> {
        let p4 = &mut self.level_4_table;
        let p3 = self.page_table_walker.next_table_mut(&mut p4[page.p4_index()])?;
        let p2 = self.page_table_walker.next_table_mut(&mut p3[page.p3_index()])?;
        Ok(&mut p2[page.p2_index()])
    }
}

impl<'a, P: PhysToVirt> Mapper<Size4KiB> for MappedPageTable<'a, P> {
    #[inline]
    unsafe fn map_to<A>(
        &mut self,
        page: Page<Size4KiB>,
        frame: Frame<Size4KiB>,
        flags: PageTableFlags,
        attr: PageTableAttribute,
        allocator: &mut A,
    ) -> Result<MapperFlush<Size4KiB>, MapToError>
    where
        A: FrameAllocator<Size4KiB>,
    {
        self.map_to_4kib(page, frame, flags, attr, allocator)
    }

    fn unmap(
        &mut self,
        page: Page<Size4KiB>,
    ) -> Result<(Frame<Size4KiB>, MapperFlush<Size4KiB>), UnmapError> {
        let entry = self.get_entry_mut(page)?;

        if !entry.flags().contains(PageTableFlags::VALID) {
            return Err(UnmapError::PageNotMapped);
        } else if entry.is_block() {
            return Err(UnmapError::ParentEntryHugePage);
        }

        let frame = Frame::from_start_address(entry.addr())
            .ok_or_else(|| UnmapError::InvalidFrameAddress(entry.addr()))?;

        entry.set_unused();
        Ok((frame, MapperFlush::new(page)))
    }

    fn get_entry(&self, page: Page<Size4KiB>) -> Result<&PageTableEntry, EntryGetError> {
        let p4 = &self.level_4_table;
        let p3 = self.page_table_walker.next_table(&p4[page.p4_index()])?;
        let p2 = self.page_table_walker.next_table(&p3[page.p3_index()])?;
        let p1 = self.page_table_walker.next_table(&p2[page.p2_index()])?;
        Ok(&p1[page.p1_index()])
    }

    fn get_entry_mut(&mut self, page: Page<Size4KiB>) -> Result<&mut PageTableEntry, EntryGetError> {
        let p4 = &mut self.level_4_table;
        let p3 = self.page_table_walker.next_table_mut(&mut p4[page.p4_index()])?;
        let p2 = self.page_table_walker.next_table_mut(&mut p3[page.p3_index()])?;
        let p1 = self.page_table_walker.next_table_mut(&mut p2[page.p2_index()])?;
        Ok(&mut p1[page.p1_index()])
    }
}

impl<'a, P: PhysToVirt> MapperAllSizes for MappedPageTable<'a, P> {
    fn translate(&self, addr: VirtAddr) -> TranslateResult {
        let p4 = &self.level_4_table;
        let p3 = match self.page_table_walker.next_table(&p4[addr.p4_index()]) {
            Ok(page_table) => page_table,
            Err(PageTableWalkError::NotMapped) => return TranslateResult::PageNotMapped,
            Err(PageTableWalkError::MappedToHugePage) => {
                panic!("level 4 entry has huge page bit set")
            }
        };
        let p2 = match self.page_table_walker.next_table(&p3[addr.p3_index()]) {
            Ok(page_table) => page_table,
            Err(PageTableWalkError::NotMapped) => return TranslateResult::PageNotMapped,
            Err(PageTableWalkError::MappedToHugePage) => {
                let frame = Frame::containing_address(p3[addr.p3_index()].addr());
                let offset = addr.as_u64() & 0o7_777_777_777;
                return TranslateResult::Frame1GiB { frame, offset };
            }
        };
        let p1 = match self.page_table_walker.next_table(&p2[addr.p2_index()]) {
            Ok(page_table) => page_table,
            Err(PageTableWalkError::NotMapped) => return TranslateResult::PageNotMapped,
            Err(PageTableWalkError::MappedToHugePage) => {
                let frame = Frame::containing_address(p2[addr.p2_index()].addr());
                let offset = addr.as_u64() & 0o7_777_777;
                return TranslateResult::Frame2MiB { frame, offset };
            }
        };

        let p1_entry = &p1[addr.p1_index()];

        if p1_entry.is_unused() {
            return TranslateResult::PageNotMapped;
        }

        let frame = match Frame::from_start_address(p1_entry.addr()) {
            Some(frame) => frame,
            None => return TranslateResult::InvalidFrameAddress(p1_entry.addr()),
        };
        let offset = u64::from(addr.page_offset());
        TranslateResult::Frame4KiB { frame, offset }
    }
}

#[derive(Debug)]
struct PageTableWalker<P: PhysToVirt> {
    phys_to_virt: P,
}

impl<P: PhysToVirt> PageTableWalker<P> {
    pub unsafe fn new(phys_to_virt: P) -> Self {
        Self { phys_to_virt }
    }

    /// Internal helper function to get a reference to the page table of the next level.
    ///
    /// Returns `PageTableWalkError::NotMapped` if the entry is unused. Returns
    /// `PageTableWalkError::MappedToHugePage` if the `HUGE_PAGE` flag is set
    /// in the passed entry.
    fn next_table<'b>(
        &self,
        entry: &'b PageTableEntry,
    ) -> Result<&'b PageTable, PageTableWalkError> {
        let page_table_ptr = self.phys_to_virt.phys_to_virt(entry.frame()?);
        let page_table: &PageTable = unsafe { &*page_table_ptr };

        Ok(page_table)
    }

    /// Internal helper function to get a mutable reference to the page table of the next level.
    ///
    /// Returns `PageTableWalkError::NotMapped` if the entry is unused. Returns
    /// `PageTableWalkError::MappedToHugePage` if the `HUGE_PAGE` flag is set
    /// in the passed entry.
    fn next_table_mut<'b>(
        &self,
        entry: &'b mut PageTableEntry,
    ) -> Result<&'b mut PageTable, PageTableWalkError> {
        let page_table_ptr = self.phys_to_virt.phys_to_virt(entry.frame()?);
        let page_table: &mut PageTable = unsafe { &mut *page_table_ptr };

        Ok(page_table)
    }

    /// Internal helper function to create the page table of the next level if needed.
    ///
    /// If the passed entry is unused, a new frame is allocated from the given allocator, zeroed,
    /// and the entry is updated to that address. If the passed entry is already mapped, the next
    /// table is returned directly.
    ///
    /// Returns `MapToError::FrameAllocationFailed` if the entry is unused and the allocator
    /// returned `None`. Returns `MapToError::ParentEntryHugePage` if the `HUGE_PAGE` flag is set
    /// in the passed entry.
    fn create_next_table<'b, A>(
        &self,
        entry: &'b mut PageTableEntry,
        allocator: &mut A,
    ) -> Result<&'b mut PageTable, PageTableCreateError>
    where
        A: FrameAllocator<Size4KiB>,
    {
        let created;

        if entry.is_unused() {
            if let Some(frame) = allocator.allocate_frame() {
                entry.set_frame(
                    frame,
                    PageTableFlags::default_table(),
                    PageTableAttribute::new(0, 0, 0),
                );
                created = true;
            } else {
                return Err(PageTableCreateError::FrameAllocationFailed);
            }
        } else {
            created = false;
        }

        let page_table = match self.next_table_mut(entry) {
            Err(PageTableWalkError::MappedToHugePage) => {
                return Err(PageTableCreateError::MappedToHugePage);
            }
            Err(PageTableWalkError::NotMapped) => panic!("entry should be mapped at this point"),
            Ok(page_table) => page_table,
        };

        if created {
            #[cfg(target_arch = "aarch64")]
            unsafe {
                crate::barrier::dsb(crate::barrier::ISHST);
            }
            page_table.clear();
        }
        Ok(page_table)
    }
}

#[derive(Debug)]
enum PageTableWalkError {
    NotMapped,
    MappedToHugePage,
}

#[derive(Debug)]
enum PageTableCreateError {
    MappedToHugePage,
    FrameAllocationFailed,
}

impl From<PageTableCreateError> for MapToError {
    fn from(err: PageTableCreateError) -> Self {
        match err {
            PageTableCreateError::MappedToHugePage => MapToError::ParentEntryHugePage,
            PageTableCreateError::FrameAllocationFailed => MapToError::FrameAllocationFailed,
        }
    }
}

impl From<FrameError> for PageTableWalkError {
    fn from(err: FrameError) -> Self {
        match err {
            FrameError::HugeFrame => PageTableWalkError::MappedToHugePage,
            FrameError::FrameNotPresent => PageTableWalkError::NotMapped,
        }
    }
}

impl From<PageTableWalkError> for EntryGetError {
    fn from(err: PageTableWalkError) -> Self {
        match err {
            PageTableWalkError::MappedToHugePage => EntryGetError::ParentEntryHugePage,
            PageTableWalkError::NotMapped => EntryGetError::PageNotMapped,
        }
    }
}

/// Trait for converting a physical address to a virtual one.
///
/// This only works if the physical address space is somehow mapped to the virtual
/// address space, e.g. at an offset.
pub trait PhysToVirt {
    /// Translate the given physical frame to a virtual page table pointer.
    fn phys_to_virt(&self, frame: Frame) -> *mut PageTable;
}

impl<T> PhysToVirt for T
where
    T: Fn(Frame) -> *mut PageTable,
{
    #[inline]
    fn phys_to_virt(&self, frame: Frame) -> *mut PageTable {
        self(frame)
    }
}
