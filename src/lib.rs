pub mod allocator;
pub mod bench;
pub mod func;
pub mod runner;

#[global_allocator]
static GLOBAL: allocator::TrackingAllocator = allocator::TrackingAllocator;

pub use allocator::{ALLOC_BYTES, ALLOC_COUNT, DEALLOC_BYTES, DEALLOC_COUNT};
pub use bench::Bench;
pub use bench::BenchPattern;
pub use bench::BenchResult;
pub use runner::measurement::Measurement;

pub use bench::instant::InstantBench;
pub use bench::scaling::ScalingBench;
pub use func::Func;
