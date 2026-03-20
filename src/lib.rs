//! A highly precise benchmarking and profiling suite for Rust.
//!
//! <details>
//! <summary>Japanese</summary>
//! Rust向けの非常に高精度なベンチマークおよびプロファイリングスイートです。
//! </details>
//!
//! `memori` provides an easy-to-use yet powerful framework to measure CPU cycles,
//! hardware instructions, execution time, and memory allocations (count & bytes) of your Rust functions.
//!
//! <details>
//! <summary>Japanese</summary>
//! `memori` は、Rust関数のCPUサイクル、ハードウェア命令数、実行時間、およびメモリの確保（回数とバイト数）を高精度に計測するための強力なフレームワークを提供します。
//! </details>
//!
//! # Features
//! - **High Precision**: Uses hardware performance counters (on Linux) or `rdtsc` for cycle-accurate measurements.
//! - **Memory Tracking**: Tracks heap allocations and deallocations per function execution.
//! - **Matrix Benchmarking**: Easily compare multiple implementations against multiple input patterns.
//!
//! <details>
//! <summary>Japanese</summary>
//!
//! # 特徴
//! - **高精度**: Linux環境ではハードウェアパフォーマンスカウンタを、その他の環境では `rdtsc` などを利用してサイクル精度の計測を行います。
//! - **メモリ追跡**: 関数の実行ごとにヒープの確保と解放を追跡します。
//! - **マトリックス計測**: 複数の実装と複数の入力パターンを掛け合わせたクロス比較が簡単にできます。
//! </details>

pub mod allocator;
pub mod func;
pub mod runner;

pub use func::{Bench, Func};
pub use runner::Runner;
pub use runner::measurement::Measurement;
