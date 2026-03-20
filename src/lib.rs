pub mod allocator;
pub mod func;
pub mod runner;

#[global_allocator]
static GLOBAL: allocator::TrackingAllocator = allocator::TrackingAllocator;

pub use allocator::{ALLOC_BYTES, ALLOC_COUNT, DEALLOC_BYTES, DEALLOC_COUNT};

pub use runner::measurement::Measurement;

pub use func::Bench;
pub use func::Func;
pub use func::output::BenchJsonEntry;
pub use func::output::BenchJsonReport;
pub use runner::Runner;
