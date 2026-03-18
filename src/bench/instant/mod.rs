use std::marker::PhantomData;
pub mod impls;

use crate::{Measurement, runner::Runner};

pub struct InstantBench<I, F, R>
where
    I: Clone,
    F: FnMut(&I) -> R,
{
    name: String,
    description: Option<String>,
    runner: Runner<I, F, R>,
    _marker: PhantomData<fn(I) -> R>,
}

impl<I, F, R> InstantBench<I, F, R>
where
    I: Clone,
    F: FnMut(&I) -> R,
{
    /// 新しい単発ベンチマーク（InstantBench）を作成します。
    ///
    /// # Arguments
    /// * `name` - ベンチマークを識別するための名前
    /// * `input` - 計測に使用する代表的な入力値
    /// * `function` - 計測対象の関数（クロージャ）
    pub fn new(name: &str, input: I, function: F) -> Self {
        Self {
            name: name.to_string(),
            description: None, // 初期状態は説明なし
            runner: Runner::new(input, function),
            _marker: PhantomData,
        }
    }

    /// ベンチマークに詳細な説明を追加するためのビルダーメソッドです。
    pub fn with_description(mut self, description: &str) -> Self {
        self.description = Some(description.to_string());
        self
    }
}
