//! # Memori
//!
//! A benchmarking and profiling suite for Rust.
//!
//! Traditional benchmarking tools focus primarily on execution time. **Memori**
//! extends this by measuring logical, environment-independent costs such as **CPU Instructions** and **Memory Allocations**.
//! It also includes a built-in HTML reporter to track performance regressions and algorithmic scaling.
//!
//! <details>
//! <summary>Japanese</summary>
//!
//! Rustのためのベンチマーク＆プロファイリングスイートです。
//!
//! 従来のベンチマークツールが実行時間を主眼に置くのに対し、**Memori（目盛り）** は物理的な時間だけでなく、
//! **CPU命令数**や**メモリアロケーション**といった「環境に依存しない論理的なコスト」も計測します。
//! また、組み込みのHTMLレポート機能により、計算量の推移やパフォーマンスの退行（リグレッション）を視覚的に追跡できます。
//! </details>
//!
//! ## Core Features
//!
//! 1. **Detailed Metrics**: Simultaneously measures CPU cycles, wall-clock time (ns), hardware instructions (Linux only), and heap allocations.
//! 2. **Scaling & Trend Analysis**:
//!    - `Bench::Scaling`: Tests algorithms against growing input sizes (`N`) to visualize Big-O complexity.
//!    - `Bench::Instant`: Tests a representative workload over time to track performance regressions.
//! 3. **CLI Progress**: Dependency-free progress animations for long-running benchmarks.
//! 4. **HTML Viewer**: Generates an interactive dashboard (`report.html`) that loads benchmark history from `target/memori/*` via a JSON manifest.
//!
//! <details>
//! <summary>Japanese</summary>
//!
//! ### 主な機能
//!
//! 1. **詳細な計測指標**: CPUサイクル、実時間(ns)、ハードウェア命令数(Linuxのみ)、ヒープの確保/解放量を同時に計測します。
//! 2. **スケーリングとトレンド分析**:
//!    - `Bench::Scaling`: 入力サイズ(`N`)を変動させ、計算量の推移（Big-O）を可視化します。
//!    - `Bench::Instant`: 固定のワークロードを継続的にテストし、パフォーマンスの退行を追跡します。
//! 3. **CLIプログレス表示**: 長時間のベンチマークでも進捗がわかる、依存関係なしの組み込みインジケータを備えています。
//! 4. **HTMLビューワー**: 履歴データ(`target/memori/*`)をマニフェスト経由で読み込むダッシュボード(`report.html`)を出力します。
//! </details>
//! ## Quick Start
//!
//! ```rust,no_run
//! use memori::{Bench, Func};
//! use std::collections::HashSet;

//! fn main() {
//!     let mut suite = Func::new("Deduplication_Battle")
//!         .with_description("Comparing allocation and time costs of deduplication strategies.")
//!         // 2. Register competing functions
//!         .add_function("HashSet", |n: &usize| {
//!             let mut set = HashSet::new();
//!             for i in 0..*n {
//!                 set.insert(i % 100);
//!             }
//!         })
//!         .add_function("Vec_Dedup", |n: &usize| {
//!             let mut vec = Vec::new();
//!             for i in 0..*n {
//!                 vec.push(i % 100);
//!             }
//!             vec.sort();
//!             vec.dedup();
//!         })
//!         // 3. Define benchmarking patterns (Scaling and Instant)
//!        .add_bench(
//!             "scaling_stress",
//!             "O(N) vs O(N log N) scaling test",
//!             Bench::Scaling(vec![100, 1000, 5000, 10000]),
//!         )
//!         .add_bench(
//!             "baseline_1k",
//!             "Continuous trend tracking for N=1000",
//!             Bench::Instant(1000),
//!         );

//!     // 4. Run the matrix and generate the HTML report!
//!     suite.run_and_save().unwrap();
//! }
//! ```

pub mod allocator;
pub mod func;
pub mod runner;

// -----------------------------------------------------------------------------
// Global Allocator Setup
// -----------------------------------------------------------------------------
// By simply depending on the `memori` crate, this tracking allocator is injected
// to intercept and measure all heap allocations globally.
//
// memoriクレートをリンクするだけで、このトラッキングアロケータが注入され、
// グローバルなヒープメモリの確保・解放が自動的に計測されるようになります。
#[global_allocator]
static GLOBAL: allocator::TrackingAllocator = allocator::TrackingAllocator;

// -----------------------------------------------------------------------------
// Public Exports (The memori Prelude)
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
