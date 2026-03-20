use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;

thread_local! {
    pub static THREAD_ALLOC_COUNT: Cell<usize> = Cell::new(0);
    pub static THREAD_ALLOC_BYTES: Cell<usize> = Cell::new(0);
    pub static THREAD_DEALLOC_COUNT: Cell<usize> = Cell::new(0);
    pub static THREAD_DEALLOC_BYTES: Cell<usize> = Cell::new(0);
}

pub struct TrackingAllocator;

unsafe impl GlobalAlloc for TrackingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let size = layout.size();
        THREAD_ALLOC_COUNT.with(|c| c.set(c.get() + 1));
        THREAD_ALLOC_BYTES.with(|c| c.set(c.get() + size));
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let size = layout.size();
        THREAD_DEALLOC_COUNT.with(|c| c.set(c.get() + 1));
        THREAD_DEALLOC_BYTES.with(|c| c.set(c.get() + size));
        unsafe { System.dealloc(ptr, layout) }
    }
}
