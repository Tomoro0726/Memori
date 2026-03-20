use serde::Serialize;

/// Absolute measurement metrics extracted from a benchmark run for CPU and memory.
///
/// In addition to "physical speed" such as execution time (cycles), it records "environment-independent logical costs"
/// such as the number of hardware instructions and memory allocation/deallocation amounts.
/// Analyzing this data helps identify algorithmic complexity and inefficient memory usage.
///
/// <details>
/// <summary>Japanese</summary>
///
/// ベンチマーク実行によって抽出された、CPUとメモリに関する絶対的な計測指標です。
///
/// 実行時間（サイクル数）といった「物理的な速度」に加えて、命令数やメモリの確保/解放量といった
/// 「環境に依存しない論理的なコスト」を記録します。
/// このデータを分析することで、アルゴリズムの計算量やメモリの非効率な使い方を特定できます。
/// </details>
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Measurement {
    /// The number of CPU cycles taken for execution.
    ///
    /// This is the closest metric to real-world execution speed but can be affected by CPU clock fluctuations
    /// and other processes (OS noise).
    ///
    /// **OS/Architecture Differences:**
    /// - **Linux & x86_64**: Strictly records CPU hardware cycles using `perf_event` or `rdtsc`.
    /// - **Other Environments (e.g., ARM macOS/Windows)**: Falls back to recording execution time in nanoseconds
    ///   if hardware cycle counting is unavailable.
    ///
    /// <details>
    /// <summary>Japanese</summary>
    ///
    /// 実行にかかったCPUサイクル数。
    ///
    /// 現実世界での実行速度に最も近い指標ですが、CPUのクロック変動や他のプロセスの影響（ノイズ）を
    /// 受ける可能性があります。
    ///
    /// **OS・アーキテクチャによる違い:**
    /// - **Linux および x86_64**: `perf_event` や `rdtsc` を用いて厳密なCPUサイクル数を記録します。
    /// - **その他の環境（ARMのmacOSやWindowsなど）**: ハードウェアレベルのサイクル取得ができない場合、
    ///   代替として実行時間（ナノ秒）が格納されます。
    /// </details>
    pub cycles: u64,

    /// The number of pure CPU hardware instructions executed.
    ///
    /// This is an ultimate stable metric, completely unaffected by CPU clock frequency or environmental noise.
    ///
    /// **OS Differences:**
    /// - Only available on **Linux** environments using `perf_event`.
    /// - On all other operating systems (Windows, macOS), this will always be `None`.
    ///
    /// <details>
    /// <summary>Japanese</summary>
    ///
    /// 実行された純粋なCPU命令数（Hardware Instructions）。
    ///
    /// CPUのクロック数や環境ノイズの影響を一切受けない、究極の安定指標です。
    ///
    /// **OSによる違い:**
    /// - **Linux環境**（`perf_event`）でのみ取得可能です。
    /// - WindowsやmacOSなど、他環境では常に `None` となります。
    /// </details>
    pub instructions: Option<u64>,

    /// The real wall-clock execution time in nanoseconds.
    ///
    /// **Feature Flag:**
    /// This field is only populated if the `real_time` Cargo feature is enabled. Otherwise, it is `None`.
    /// Unlike `cycles`, this represents absolute time, making it easier to grasp the actual latency.
    ///
    /// <details>
    /// <summary>Japanese</summary>
    ///
    /// ナノ秒単位での実実行時間（Wall-clock time）。
    ///
    /// **機能フラグ:**
    /// Cargoの `real_time` フィーチャーが有効な場合のみ記録され、それ以外の場合は `None` になります。
    /// `cycles` とは異なり絶対的な時間を表すため、実際のレイテンシを把握するのに役立ちます。
    /// </details>
    pub time_ns: Option<u64>,

    /// The number of times heap memory allocation was requested from the OS.
    ///
    /// <details>
    /// <summary>Japanese</summary>
    ///
    /// OSに対してヒープメモリの確保（Alloc）を要求した回数。
    /// </details>
    pub alloc_count: usize,

    /// The total amount of heap memory requested from the OS, in bytes.
    ///
    /// <details>
    /// <summary>Japanese</summary>
    ///
    /// OSに対して要求したヒープメモリの総確保量（バイト数）。
    /// </details>
    pub alloc_bytes: usize,

    /// The number of times heap memory deallocation was requested from the OS.
    ///
    /// **Profiling Tip:** If both `alloc_count` and `dealloc_count` are abnormally high,
    /// it indicates memory thrashing (unnecessary create-and-destroy cycles) within the function,
    /// which is highly likely to be a performance bottleneck.
    ///
    /// <details>
    /// <summary>Japanese</summary>
    ///
    /// OSに対してヒープメモリの解放（Dealloc）を要求した回数。
    ///
    /// **プロファイリングのヒント:** `alloc_count` と `dealloc_count` が共に異常に高い場合、
    /// 関数内で「作っては捨てる」無駄な処理（Thrashing）が発生しており、
    /// パフォーマンス低下のボトルネックになっている可能性が高いです。
    /// </details>
    pub dealloc_count: usize,

    /// The total amount of heap memory deallocated to the OS, in bytes.
    ///
    /// <details>
    /// <summary>Japanese</summary>
    ///
    /// OSに対して解放したヒープメモリの総量（バイト数）。
    /// </details>
    pub dealloc_bytes: usize,
}

impl Measurement {
    /// Creates a new measurement data instance.
    ///
    /// <details>
    /// <summary>Japanese</summary>
    ///
    /// 新しい計測データを生成します。
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
        }
    }

    /// Calculates the "net memory increase" in bytes by subtracting the deallocated memory from the allocated memory.
    ///
    /// - **If the value is `0`**: All memory allocated within the function was cleanly freed.
    /// - **If the value is `> 0`**: The function either returned the allocated data or a memory leak occurred.
    ///
    /// <details>
    /// <summary>Japanese</summary>
    ///
    /// 確保したメモリ量から解放したメモリ量を差し引いた「正味のメモリ増加量 (Net Memory)」をバイト単位で計算します。
    ///
    /// - **値が `0` の場合:** 関数内で確保されたメモリはすべて綺麗に解放されています。
    /// - **値が `0` より大きい場合:** 関数がデータを返却したか、内部でメモリリークが発生しています。
    /// </details>
    pub fn net_bytes(&self) -> isize {
        (self.alloc_bytes as isize) - (self.dealloc_bytes as isize)
    }

    /// Calculates the net number of allocations (allocations minus deallocations).
    ///
    /// <details>
    /// <summary>Japanese</summary>
    ///
    /// 正味のメモリ確保回数 (Net Allocations) を計算します。
    /// </details>
    pub fn net_allocs(&self) -> isize {
        (self.alloc_count as isize) - (self.dealloc_count as isize)
    }
}
