//! Orchestration and definition of benchmark suites.

pub mod output;

/// The input pattern for a benchmark.
pub enum Bench<I> {
    /// Benchmark with a single representative value.
    Instant(I),
    /// Benchmark with multiple values to measure scaling and trends.
    Scaling(Vec<I>),
}

/// A single benchmark scenario (pattern).
pub struct BenchPattern<I> {
    /// The name of the benchmark pattern.
    pub name: String,
    /// A description of what this pattern tests.
    pub description: String,
    /// The input values and pattern type.
    pub input: Bench<I>,
}

/// The main orchestrator for benchmarking.
///
/// `Func` allows you to register multiple competing functions and benchmark scenarios.
/// It executes a full matrix of all functions against all patterns.
pub struct Func<I>
where
    I: Clone,
{
    pub name: String,
    pub description: Option<String>,
    pub functions: Vec<(String, Box<dyn FnMut(&I)>)>,
    pub patterns: Vec<BenchPattern<I>>,
}

impl<I> Func<I>
where
    I: Clone,
{
    /// Creates a new benchmark suite.
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            description: None,
            functions: Vec::new(),
            patterns: Vec::new(),
        }
    }

    /// Adds a global description to the benchmark suite.
    pub fn with_description(mut self, description: &str) -> Self {
        self.description = Some(description.to_string());
        self
    }

    /// Registers a target function to be benchmarked.
    pub fn add_function<F, R>(mut self, name: &str, mut func: F) -> Self
    where
        F: FnMut(&I) -> R + 'static,
    {
        let wrapped = move |i: &I| {
            let _ = std::hint::black_box(func(i));
        };
        self.functions.push((name.to_string(), Box::new(wrapped)));
        self
    }

    /// Registers a benchmark scenario.
    pub fn add_bench(mut self, name: &str, description: &str, input: Bench<I>) -> Self {
        self.patterns.push(BenchPattern {
            name: name.to_string(),
            description: description.to_string(),
            input,
        });
        self
    }
}
