# Memori ⚖️

A highly precise, multi-dimensional benchmarking and profiling suite for Rust.

## Features

- Multi-dimensional metrics: CPU cycles, wall-clock time, hardware instructions (Linux only), heap allocations/deallocations
- Scaling vs Trend Analysis: `Bench::Scaling` for Big-O complexity, `Bench::Instant` for regression tracking
- Beautiful CLI DX: Built-in progress animations
- Standalone HTML Viewer: Generates interactive dashboard (`report.html`) for easy history inspection

## Quick Start

```rust
use memori::{Func, Bench};
use std::collections::HashSet;

fn main() -> Result<(), Box<dyn std::error::Error> {
    // 1. Create a new benchmark suite
    let mut suite = Func::new("Deduplication_Battle")
        .with_description("Comparing allocation and time costs of deduplication strategies.");

    // 2. Register competing functions
    suite = suite
        .add_function("HashSet", |n: &usize| {
            let mut set = HashSet::new();
            for i in 0..*n { set.insert(i % 100); }
        })
        .add_function("Vec_Dedup", |n: &usize| {
            let mut vec = Vec::new();
            for i in 0..*n { vec.push(i % 100); }
            vec.sort();
            vec.dedup();
        });

    // 3. Define benchmarking patterns (Scaling and Instant)
    suite = suite
        .add_bench(
            "scaling_stress",
            "O(N) vs O(N log N) scaling test",
            Bench::Scaling(vec![100, 1000, 5000, 10000])
        )
        .add_bench(
            "baseline_1k",
            "Continuous trend tracking for N=1000",
            Bench::Instant(1000)
        );

    // 4. Run the matrix and generate the HTML report!
    suite.run_and_save()?;

    Ok(())
}
```

## Usage

- Add `memori` to your `Cargo.toml` dependencies
- Write benchmarks using `Func` and `Bench` APIs
- Run your benchmarks and view results in `report.html`

## License

MIT
