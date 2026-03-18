use crate::Measurement;

pub mod instant;
pub mod scaling;

///このベンチマークソフトがサポートするベンチマークのパターン
pub trait Bench<I>
where
    I: Clone,
{
    ///このベンチマークの名前
    fn name(&self) -> &String;

    ///このベンチマークの説明
    fn description(&self) -> Option<&String>;

    ///どんなベンチマークなのかを返す
    fn pattern(&self) -> BenchPattern;

    ///実行結果を返す
    fn run(&mut self) -> BenchResult<I>;
}

pub enum BenchPattern {
    Instant,
    Scaling,
}

pub enum BenchResult<I>
where
    I: Clone,
{
    Instant((I, Measurement)),
    Scaling(Vec<(I, Measurement)>),
}
