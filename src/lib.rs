//! # Tenbin ⚖️
//!
//! A highly precise, multi-dimensional benchmarking and profiling suite for Rust.
//!
//! While traditional benchmarking tools focus solely on execution time, **Tenbin** acts as a true "balance scale" (天秤),
//! weighing not just physical time, but also logical, environment-independent costs such as **CPU Instructions** and **Memory Allocations**.
//!
//! Combined with its zero-config, standalone React-based HTML reporter, Tenbin provides the ultimate developer experience (DX)
//! for tracking performance regressions and algorithmic scaling.
//!
//! <details>
//! <summary>Japanese</summary>
//!
//! Rustのための高精度・多次元ベンチマーク＆プロファイリングスイートです。
//!
//! 従来のベンチマークツールが実行時間のみに焦点を当てているのに対し、**Tenbin（天秤）** は物理的な時間だけでなく、
//! **CPU命令数**や**メモリアロケーション**といった「環境に依存しない論理的なコスト」も精密に計量します。
//! ゼロコンフィグで出力されるReactベースのスタンドアロンHTMLレポートと組み合わせることで、
//! アルゴリズムの計算量推移やリグレッションを追跡するための究極の開発者体験（DX）を提供します。
//! </details>
//!
//! ## Core Concepts
//!
//! 1. **Multi-Dimensional Metrics**: Measures CPU Cycles, Wall-clock time (ns), Hardware Instructions (Linux only), and Heap Allocations/Deallocations simultaneously.
//! 2. **Scaling vs Trend Analysis**:
//!    - `Bench::Scaling`: Tests algorithms against growing input sizes (`N`) to visualize Big-O complexity.
//!    - `Bench::Instant`: Tests a representative workload repeatedly over time to track performance regressions (CodSpeed-like trend graphs).
//! 3. **Beautiful CLI DX**: Built-in, dependency-free progress animations (`\r` rewriting) to keep you informed during heavy stress tests.
//! 4. **Standalone HTML Viewer**: Generates a rich, interactive dashboard (`report.html`) and loads benchmark JSON from `target/tenbin/*` folders via a manifest, making history files easy to inspect and prune.
//!
//! <details>
//! <summary>Japanese</summary>
//!
//! ### コアコンセプト
//!
//! 1. **多次元の計測指標**: CPUサイクル、実時間(ns)、ハードウェア命令数(Linuxのみ)、ヒープアロケーション/デアロケーションを同時に計測します。
//! 2. **スケーリングとトレンド分析**:
//!    - `Bench::Scaling`: 入力サイズ(`N`)を増大させながらテストし、Big-O記法的な計算量の推移を可視化します。
//!    - `Bench::Instant`: 代表的なワークロードを継続的にテストし、パフォーマンスの悪化（リグレッション）をトレンドグラフとして追跡します。
//! 3. **美しいCLI体験**: 依存関係なしの組み込みプログレスアニメーションにより、重いストレステスト中も進捗を美しく表示します。
//! 4. **スタンドアロンHTMLビューワー**: リッチなダッシュボード(`report.html`)を生成し、`target/tenbin/*` 配下のJSONをmanifest経由で読み込むため、履歴ファイルを人手で整理しやすくなります。
//! </details>
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use tenbin::{Func, Bench};
//! use std::collections::HashSet;
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // 1. Create a new benchmark suite
//!     let mut suite = Func::new("Deduplication_Battle")
//!         .with_description("Comparing allocation and time costs of deduplication strategies.");
//!
//!     // 2. Register competing functions
//!     suite = suite
//!         .add_function("HashSet", |n: &usize| {
//!             let mut set = HashSet::new();
//!             for i in 0..*n { set.insert(i % 100); }
//!         })
//!         .add_function("Vec_Dedup", |n: &usize| {
//!             let mut vec = Vec::new();
//!             for i in 0..*n { vec.push(i % 100); }
//!             vec.sort();
//!             vec.dedup();
//!         });
//!
//!     // 3. Define benchmarking patterns (Scaling and Instant)
//!     suite = suite
//!         .add_bench(
//!             "scaling_stress",
//!             "O(N) vs O(N log N) scaling test",
//!             Bench::Scaling(vec![100, 1000, 5000, 10000])
//!         )
//!         .add_bench(
//!             "baseline_1k",
//!             "Continuous trend tracking for N=1000",
//!             Bench::Instant(1000)
//!         );
//!
//!     // 4. Run the matrix and generate the HTML report!
//!     suite.run_and_save()?;
//!
//!     Ok(())
//! }
//! ```

pub mod allocator;
pub mod func;
pub mod runner;

// -----------------------------------------------------------------------------
// Global Allocator Setup
// -----------------------------------------------------------------------------
// By simply depending on the `tenbin` crate, this tracking allocator is injected
// to intercept and measure all heap allocations globally.
//
// Tenbinクレートをリンクするだけで、このトラッキングアロケータが注入され、
// グローバルなヒープメモリの確保・解放が自動的に計測されるようになります。
#[global_allocator]
static GLOBAL: allocator::TrackingAllocator = allocator::TrackingAllocator;

// -----------------------------------------------------------------------------
// Public Exports (The Tenbin Prelude)
// -----------------------------------------------------------------------------

// Core Orchestration
pub use func::Bench;
pub use func::Func;

// Measurement Metrics
pub use runner::measurement::Measurement;

// Internal Runner (Exported for advanced users who want to bypass `Func`)
pub use runner::Runner;

// Output Structures (Exported for programmatic consumption of `run_all`)
pub use func::output::BenchJsonEntry;
pub use func::output::BenchJsonReport;
pub use func::output::FuncMetadata;
pub use func::output::PatternMetadata;

// Low-level Global Counters (Exported in case users need direct access)
pub use allocator::{ALLOC_BYTES, ALLOC_COUNT, DEALLOC_BYTES, DEALLOC_COUNT};
