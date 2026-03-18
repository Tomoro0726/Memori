use crate::ScalingBench;
use crate::bench::{Bench, BenchPattern, BenchResult};
use crate::runner::Runner;

impl<I, F, R> Bench<I> for ScalingBench<I, F, R>
where
    I: Clone,
    F: FnMut(&I) -> R,
{
    /// ベンチマークの名前を返します
    fn name(&self) -> &String {
        &self.name
    }

    /// ベンチマークの説明を返します
    fn description(&self) -> Option<&String> {
        self.description.as_ref()
    }

    /// これが Scaling パターンであることを伝えます
    fn pattern(&self) -> BenchPattern {
        BenchPattern::Scaling
    }

    fn run(&mut self) -> BenchResult<I> {
        let mut results = Vec::new();

        for input in &self.inputs {
            let mut runner = Runner::new(input.clone(), &mut self.function);

            let measurement = runner.run();

            results.push((input.clone(), measurement));
        }

        BenchResult::Scaling(results)
    }
}
