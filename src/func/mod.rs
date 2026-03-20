use std::collections::BTreeMap;

use crate::{BenchJsonEntry, BenchJsonReport, Runner};
pub mod output;

/// The input pattern for a benchmark.
///
/// Defines whether the benchmark should be run with a single representative value (`Instant`)
/// or a series of values to observe performance scaling (`Scaling`).
///
/// <details>
/// <summary>Japanese</summary>
///
/// 計測の入力パターンです。
///
/// 単一の代表値で計測する（`Instant`）か、複数の値を与えてパフォーマンスの推移を計測する（`Scaling`）かを定義します。
/// </details>
pub enum Bench<I> {
    /// Benchmark with a single representative value.
    ///
    /// <details>
    /// <summary>Japanese</summary>
    /// 単一の代表値で計測します。
    /// </details>
    Instant(I),

    /// Benchmark with multiple values to measure scaling and trends.
    ///
    /// <details>
    /// <summary>Japanese</summary>
    /// 複数の値でスケーリング（推移）を計測します。
    /// </details>
    Scaling(Vec<I>),
}

/// A single benchmark scenario (pattern).
///
/// <details>
/// <summary>Japanese</summary>
///
/// 1つの計測シナリオ（パターン）を表す構造体です。
/// </details>
pub struct BenchPattern<I> {
    /// The name of the benchmark pattern.
    pub name: String,
    /// A human-readable description of what this pattern tests.
    pub description: String,
    /// The input values and pattern type (`Instant` or `Scaling`).
    pub input: Bench<I>,
}

/// The main orchestrator for benchmarking.
///
/// `Func` allows you to register multiple competing functions and multiple benchmark scenarios (patterns).
/// It then automatically executes a full matrix (cross-product) of all functions against all patterns.
///
/// <details>
/// <summary>Japanese</summary>
///
/// ベンチマーク実行のメインオーケストレーターです。
///
/// 複数の競合する関数（ライバル）と、複数の計測シナリオ（パターン）を登録できます。
/// その後、登録されたすべての関数に対して、すべてのパターンを掛け合わせる「マトリックス実行」を自動で行います。
/// </details>
pub struct Func<I>
where
    I: Clone,
{
    name: String,
    description: Option<String>,
    /// 登録された複数の関数（ライバルたち）
    functions: Vec<(String, Box<dyn FnMut(&I)>)>,
    /// 登録された計測シナリオ
    patterns: Vec<BenchPattern<I>>,
}

impl<I> Func<I>
where
    I: Clone,
{
    /// Creates a new benchmark suite.
    ///
    /// <details>
    /// <summary>Japanese</summary>
    ///
    /// 新しい計測スイートを作成します。
    /// </details>
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            description: None,
            functions: Vec::new(),
            patterns: Vec::new(),
        }
    }

    /// Adds a global description to the benchmark suite.
    ///
    /// <details>
    /// <summary>Japanese</summary>
    ///
    /// 計測スイート全体の説明文を追加します。
    /// </details>
    pub fn with_description(mut self, description: &str) -> Self {
        self.description = Some(description.to_string());
        self
    }

    /// Registers a target function to be benchmarked.
    ///
    /// You can chain this method to register multiple implementations for comparison.
    /// The return type `R` of the provided closure is internally discarded to unify the function signatures.
    ///
    /// <details>
    /// <summary>Japanese</summary>
    ///
    /// 計測対象の関数を追加します。
    ///
    /// このメソッドをチェーンして呼び出すことで、比較したい複数の実装（ライバルたち）を登録できます。
    /// 内部でクロージャの戻り値 `R` は破棄され、異種な戻り値を持つ関数でも単一のコレクションで管理できるよう型消去が行われます。
    /// </details>
    pub fn add_function<F, R>(mut self, name: &str, mut func: F) -> Self
    where
        F: FnMut(&I) -> R + 'static,
    {
        // 戻り値 R を捨てて、型を統一する
        let wrapped = move |i: &I| {
            let _ = func(i);
        };
        self.functions.push((name.to_string(), Box::new(wrapped)));
        self
    }

    /// Registers a benchmark scenario (either an `Instant` or `Scaling` pattern).
    ///
    /// <details>
    /// <summary>Japanese</summary>
    ///
    /// 計測シナリオ（代表値またはスケーリング）を追加します。
    /// </details>
    pub fn add_bench(mut self, name: &str, description: &str, input: Bench<I>) -> Self {
        self.patterns.push(BenchPattern {
            name: name.to_string(),
            description: description.to_string(),
            input,
        });
        self
    }
}
