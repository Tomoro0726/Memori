use crate::{Bench, BenchPattern, BenchResult, bench::instant::InstantBench};

impl<I, F, R> Bench<I> for InstantBench<I, F, R>
where
    I: Clone,
    F: FnMut(&I) -> R,
{
    fn name(&self) -> &String {
        &self.name
    }

    fn description(&self) -> Option<&String> {
        self.description.as_ref()
    }

    fn pattern(&self) -> BenchPattern {
        BenchPattern::Instant
    }

    fn run(&mut self) -> BenchResult<I> {
        let measurement = self.runner.run();
        BenchResult::Instant((self.runner.input().clone(), measurement))
    }
}
