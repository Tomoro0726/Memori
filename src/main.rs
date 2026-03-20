use memori::{Bench, Func};
use std::collections::HashSet;

fn main() {
    let mut suite = Func::new("Deduplication_Battle")
        .with_description("Comparing allocation and time costs of deduplication strategies.")
        // 2. Register competing functions
        .add_function("HashSet", |n: &usize| {
            let mut set = HashSet::new();
            for i in 0..*n {
                set.insert(i % 100);
            }
        })
        .add_function("Vec_Dedup", |n: &usize| {
            let mut vec = Vec::new();
            for i in 0..*n {
                vec.push(i % 100);
            }
            vec.sort();
            vec.dedup();
        })
        // 3. Define benchmarking patterns (Scaling and Instant)
        .add_bench(
            "scaling_stress",
            "O(N) vs O(N log N) scaling test",
            Bench::Scaling(vec![100, 1000, 5000, 10000]),
        )
        .add_bench(
            "baseline_1k",
            "Continuous trend tracking for N=1000",
            Bench::Instant(1000),
        );

    // 4. Run the matrix and generate the HTML report!
    suite.run_and_save().unwrap();
}
