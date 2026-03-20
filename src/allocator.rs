//! Global memory allocator for tracking allocations during benchmarks.

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;

thread_local! {
    pub static THREAD_ALLOC_COUNT: Cell<usize> = const { Cell::new(0) };
    pub static THREAD_ALLOC_BYTES: Cell<usize> = const { Cell::new(0) };
    pub static THREAD_DEALLOC_COUNT: Cell<usize> = const { Cell::new(0) };
    pub static THREAD_DEALLOC_BYTES: Cell<usize> = const { Cell::new(0) };
    static IN_ALLOCATOR: Cell<bool> = const { Cell::new(false) };
}

/// A global allocator that tracks memory allocation statistics per thread.
///
/// It wraps the standard `System` allocator and records the number and size
/// of allocations and deallocations.
pub struct TrackingAllocator;

unsafe impl GlobalAlloc for TrackingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let size = layout.size();

        let _ = IN_ALLOCATOR.try_with(|in_alloc| {
            if !in_alloc.get() {
                in_alloc.set(true);
                let _ = THREAD_ALLOC_COUNT.try_with(|c| c.set(c.get().wrapping_add(1)));
                let _ = THREAD_ALLOC_BYTES.try_with(|c| c.set(c.get().wrapping_add(size)));
                in_alloc.set(false);
            }
        });

        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let size = layout.size();

        let _ = IN_ALLOCATOR.try_with(|in_alloc| {
            if !in_alloc.get() {
                in_alloc.set(true);
                let _ = THREAD_DEALLOC_COUNT.try_with(|c| c.set(c.get().wrapping_add(1)));
                let _ = THREAD_DEALLOC_BYTES.try_with(|c| c.set(c.get().wrapping_add(size)));
                in_alloc.set(false);
            }
        });

        unsafe { System.dealloc(ptr, layout) }
    }
}
