//! Global memory allocator for tracking allocations during benchmarks.
//!
//! <details>
//! <summary>Japanese</summary>
//! ベンチマーク中のメモリ割り当てを追跡するためのグローバルメモリアロケーター。
//! </details>

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

pub static ALLOC_COUNT: AtomicUsize = AtomicUsize::new(0);
pub static ALLOC_BYTES: AtomicUsize = AtomicUsize::new(0);
pub static DEALLOC_COUNT: AtomicUsize = AtomicUsize::new(0);
pub static DEALLOC_BYTES: AtomicUsize = AtomicUsize::new(0);

/// A global allocator that tracks memory allocation statistics globally.
///
/// <details>
/// <summary>Japanese</summary>
/// メモリ割り当て統計をグローバルに追跡するアロケーター。
/// アトミック変数を使用することで、安全かつ超低オーバーヘッドで動作します。
/// </details>
pub struct TrackingAllocator;

unsafe impl GlobalAlloc for TrackingAllocator {
    #[inline]
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        ALLOC_COUNT.fetch_add(1, Ordering::Relaxed);
        ALLOC_BYTES.fetch_add(layout.size(), Ordering::Relaxed);
        unsafe { System.alloc(layout) }
    }

    #[inline]
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        DEALLOC_COUNT.fetch_add(1, Ordering::Relaxed);
        DEALLOC_BYTES.fetch_add(layout.size(), Ordering::Relaxed);
        unsafe { System.dealloc(ptr, layout) }
    }
}
