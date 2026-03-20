//! Data structures for storing benchmark results.
//!
//! <details>
//! <summary>Japanese</summary>
//! ベンチマーク結果を保存するためのデータ構造。
//! </details>

use serde::Deserialize;
use serde::Serialize;

/// Absolute measurement metrics extracted from a benchmark run for CPU and memory.
///
/// In addition to execution time (cycles/nanoseconds), it records environment-independent
/// logical costs such as the number of hardware instructions and memory allocations.
///
/// <details>
/// <summary>Japanese</summary>
/// ベンチマーク実行から抽出された、CPUとメモリに関する絶対的な計測指標。
/// 実行時間（サイクル数やナノ秒）に加えて、ハードウェア命令数やメモリ割り当てといった環境に依存しない論理コストを記録します。
/// </details>
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Measurement {
    /// The number of CPU cycles taken for execution.
    ///
    /// - **Linux & x86_64**: Records CPU hardware cycles using `perf_event` or `rdtsc`.
    /// - **Other Environments**: Falls back to 0 if hardware cycle counting is unavailable.
    ///
    /// <details>
    /// <summary>Japanese</summary>
    /// 実行にかかったCPUサイクル数。
    ///
    /// - **Linux および x86_64**: `perf_event` や `rdtsc` を使用してCPUハードウェアサイクルを記録します。
    /// - **その他の環境**: 計測できない場合は0になります。
    /// </details>
    pub cycles: u64,

    /// The number of pure CPU hardware instructions executed.
    ///
    /// Only available on Linux environments using `perf_event`. On all other
    /// operating systems, this will always be `None`.
    ///
    /// <details>
    /// <summary>Japanese</summary>
    /// 実行された純粋なハードウェア命令数。
    /// Linux環境（`perf_event`）でのみ利用可能であり、それ以外のOSでは常に `None` となります。
    /// </details>
    pub instructions: Option<u64>,

    /// The real wall-clock execution time in nanoseconds.
    ///
    /// Only recorded if the `real_time` feature is enabled. Otherwise, this is `None`.
    ///
    /// <details>
    /// <summary>Japanese</summary>
    /// ナノ秒単位での実実行時間（Wall-clock time）。
    /// `real_time` フィーチャーが有効な場合のみ記録され、それ以外は `None` となります。
    /// </details>
    pub time_ns: Option<u64>,

    /// The number of times heap memory allocation was requested.
    ///
    /// <details>
    /// <summary>Japanese</summary>
    /// ヒープメモリの確保（Alloc）が要求された回数。
    /// </details>
    pub alloc_count: usize,

    /// The total amount of heap memory requested, in bytes.
    ///
    /// <details>
    /// <summary>Japanese</summary>
    /// 要求されたヒープメモリの総確保量（バイト数）。
    /// </details>
    pub alloc_bytes: usize,

    /// The number of times heap memory deallocation was requested.
    ///
    /// <details>
    /// <summary>Japanese</summary>
    /// ヒープメモリの解放（Dealloc）が要求された回数。
    /// </details>
    pub dealloc_count: usize,

    /// The total amount of heap memory deallocated, in bytes.
    ///
    /// <details>
    /// <summary>Japanese</summary>
    /// 解放されたヒープメモリの総量（バイト数）。
    /// </details>
    pub dealloc_bytes: usize,

    /// Net memory increase (alloc_bytes - dealloc_bytes).
    ///
    /// <details>
    /// <summary>Japanese</summary>
    /// 正味のメモリ増加量（確保バイト数 - 解放バイト数）。
    /// </details>
    pub net_bytes: isize,
}

impl Measurement {
    /// Creates a new measurement data instance.
    ///
    /// <details>
    /// <summary>Japanese</summary>
    /// 新しい計測データインスタンスを作成します。
    /// </details>
    pub fn new(
        cycles: u64,
        instructions: Option<u64>,
        time_ns: Option<u64>,
        alloc_count: usize,
        alloc_bytes: usize,
        dealloc_count: usize,
        dealloc_bytes: usize,
    ) -> Self {
        Self {
            cycles,
            instructions,
            time_ns,
            alloc_count,
            alloc_bytes,
            dealloc_count,
            dealloc_bytes,
            net_bytes: (alloc_bytes as isize) - (dealloc_bytes as isize),
        }
    }

    /// Calculates the net number of allocations (allocations minus deallocations).
    ///
    /// <details>
    /// <summary>Japanese</summary>
    /// 正味のメモリ確保回数（確保回数 - 解放回数）を計算します。
    /// </details>
    pub fn net_allocs(&self) -> isize {
        (self.alloc_count as isize) - (self.dealloc_count as isize)
    }
}
