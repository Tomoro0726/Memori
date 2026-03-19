/// ベンチマーク実行によって抽出された、CPUとメモリに関する絶対的な計測指標です。
///
/// 実行時間（サイクル数）といった「物理的な速度」に加えて、
/// 命令数やメモリの確保/解放量といった「環境に依存しない論理的なコスト」を記録します。
/// このデータを分析することで、アルゴリズムの計算量やメモリの非効率な使い方を特定できます。
///
use serde::Serialize;
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Measurement {
    /// 実行にかかったCPUサイクル数。
    ///
    /// 現実世界での実行速度に最も近い指標ですが、CPUのクロック変動や
    /// 他のプロセスの影響（ノイズ）を受ける可能性があります。
    /// ※ Windows/macOS 環境では代替としてナノ秒が格納される場合があります。
    pub cycles: u64,

    /// 実行された純粋なCPU命令数（Hardware Instructions）。
    ///
    /// CPUのクロック数や環境ノイズの影響を一切受けない、究極の安定指標です。
    /// ※ Linux環境（`perf_event`）でのみ取得可能。他環境では `None` となります。
    pub instructions: Option<u64>,

    pub time_ns: Option<u64>,

    /// OSに対してヒープメモリの確保（Alloc）を要求した回数。
    pub alloc_count: usize,

    /// OSに対して要求したヒープメモリの総確保量（バイト数）。
    pub alloc_bytes: usize,

    /// OSに対してヒープメモリの解放（Dealloc）を要求した回数。
    ///
    /// **プロファイリングのヒント:** `alloc_count` と `dealloc_count` が共に異常に高い場合、
    /// 関数内で「作っては捨てる」無駄な処理（Thrashing）が発生しており、
    /// パフォーマンス低下のボトルネックになっている可能性が高いです。
    pub dealloc_count: usize,

    /// OSに対して解放したヒープメモリの総量（バイト数）。
    pub dealloc_bytes: usize,
}

impl Measurement {
    /// 新しい計測データを生成します。
    pub fn new(
        cycles: u64,
        instructions: Option<u64>,
        time_ns: Option<u64>, // ← 追加
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

    /// 確保したメモリ量から解放したメモリ量を差し引いた「正味のメモリ増加量 (Net Memory)」をバイト単位で計算します。
    ///
    /// - **値が `0` の場合:** 関数内で確保されたメモリはすべて綺麗に解放されています。
    /// - **値が `0` より大きい場合:** 関数がデータを返却したか、内部でメモリリークが発生しています。
    pub fn net_bytes(&self) -> isize {
        (self.alloc_bytes as isize) - (self.dealloc_bytes as isize)
    }

    /// 正味のメモリ確保回数 (Net Allocations) を計算します。
    pub fn net_allocs(&self) -> isize {
        (self.alloc_count as isize) - (self.dealloc_count as isize)
    }
}
