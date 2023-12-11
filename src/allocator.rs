pub mod bump;
pub mod fixed_size_block;
pub mod linked_list;

use alloc::alloc::{GlobalAlloc, Layout};
use core::ptr::null_mut;

use linked_list_allocator::LockedHeap;
use x86_64::structures::paging::mapper::MapToError;
use x86_64::structures::paging::{FrameAllocator, Mapper, Page, PageTableFlags, Size4KiB};
use x86_64::VirtAddr;

use bump::BumpAllocator;
use fixed_size_block::FixedSizeBlockAllocator;
use linked_list::LinkedListAllocator;

#[global_allocator]
static ALLOCATOR: Locked<FixedSizeBlockAllocator> = Locked::new(FixedSizeBlockAllocator::new());

// static ALLOCATOR: Locked<LinkedListAllocator> = Locked::new(LinkedListAllocator::new());

// static ALLOCATOR: Locked<BumpAllocator> = Locked::new(BumpAllocator::new());

// static ALLOCATOR: LockedHeap = LockedHeap::empty();

pub const HEAP_START: usize = 0x_4444_4444_0000;
pub const HEAP_SIZE: usize = 100 * 1024; // 100 KiB

pub struct Dummy;

unsafe impl GlobalAlloc for Dummy {
    unsafe fn alloc(&self, _layout: Layout) -> *mut u8 {
        null_mut()
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        panic!("dealloc should be never called")
    }
}

pub fn init_heap(
    mapper: &mut impl Mapper<Size4KiB>,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) -> Result<(), MapToError<Size4KiB>> {
    let page_range = {
        let heap_start = VirtAddr::new(HEAP_START as u64);
        let heap_end = heap_start + HEAP_SIZE - 1_u64;
        Page::range_inclusive(
            Page::containing_address(heap_start),
            Page::containing_address(heap_end),
        )
    };

    for page in page_range {
        let frame = frame_allocator
            .allocate_frame()
            .ok_or(MapToError::FrameAllocationFailed)?;
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
        unsafe { mapper.map_to(page, frame, flags, frame_allocator)?.flush() };
    }

    unsafe {
        ALLOCATOR.lock().init(HEAP_START, HEAP_SIZE);
    }

    Ok(())
}

/// A wrapper around spin::Mutex to permit trait implementations.
pub struct Locked<A> {
    inner: spin::Mutex<A>,
}

impl<A> Locked<A> {
    pub const fn new(inner: A) -> Self {
        Locked {
            inner: spin::Mutex::new(inner),
        }
    }

    pub fn lock(&self) -> spin::MutexGuard<A> {
        self.inner.lock()
    }
}

/// Align the given address `addr` upwards to alignment `align`.
///
/// Requires that `align` is a power of two.
fn align_up(addr: usize, align: usize) -> usize {
    // using bit mask
    // Example:
    // align = 0b00010000 since it is a power of two
    // align - 1 = 0b00001111
    // !(align - 1) = 0b11110000 => a bit mask that aligns addr DOWNWARDS using &
    // using (addr + align - 1) aligns addr UPWARDS while already aligned addr is unaffected
    (addr + align - 1) & !(align - 1)
}
