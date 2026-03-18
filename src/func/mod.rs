use crate::bench::{Bench, BenchResult};
use std::marker::PhantomData;
pub mod output;

/// 関数ごとに作られる構造体
pub struct Func<I>
where
    I: Clone,
{
    name: String,
    description: Option<String>,
    // 具象型 B ではなく、トレイトオブジェクトの Vec に変更
    benches: Vec<Box<dyn Bench<I>>>,
    _marker: PhantomData<I>, // I だけ見守る
}

impl<I> Func<I>
where
    I: Clone,
{
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            description: None,
            benches: Vec::new(),
            _marker: PhantomData,
        }
    }

    pub fn with_description(mut self, description: &str) -> Self {
        self.description = Some(description.to_string());
        self
    }

    /// どんな Bench 実装（InstantでもScalingでも）でも受け入れ可能にする
    pub fn add_bench(mut self, bench: impl Bench<I> + 'static) -> Self {
        self.benches.push(Box::new(bench));
        self
    }

    pub fn run_all(&mut self) -> Vec<BenchResult<I>> {
        let mut results = Vec::new();
        for bench in &mut self.benches {
            results.push(bench.run());
        }
        results
    }
}
