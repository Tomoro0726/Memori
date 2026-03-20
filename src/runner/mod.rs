use crate::Measurement;
pub mod measurement;
use std::sync::atomic::Ordering;

/// The minimal execution unit for running benchmarks.
///
/// It encapsulates the target function and a specific input value, providing highly accurate
/// measurements of CPU cycles, execution time, and memory allocations.
///
/// <details>
/// <summary>Japanese</summary>
///
/// ベンチマーク実行の最小単位です。
///
/// ターゲットとなる関数と特定の入力値をカプセル化し、CPUサイクル数、実行時間、
/// およびメモリアロケーションの高精度な計測を提供します。
/// </details>
pub struct Runner<I, F, R>
where
    I: Clone,
    F: FnMut(&I) -> R,
{
    input: I,
    function: F,
}

impl<I, F, R> Runner<I, F, R>
where
    I: Clone,
    F: FnMut(&I) -> R,
{
    /// Creates a new `Runner` instance.
    ///
    /// # Arguments
    /// * `input` - The input value to be passed to the benchmark function.
    /// * `function` - The closure or function to be benchmarked.
    ///
    /// <details>
    /// <summary>Japanese</summary>
    ///
    /// 新しい `Runner` インスタンスを作成します。
    ///
    /// # 引数
    /// * `input` - ベンチマーク関数に渡される入力値。
    /// * `function` - ベンチマーク対象のクロージャまたは関数。
    /// </details>
    pub fn new(input: I, function: F) -> Self {
        Runner { input, function }
    }

    /// Returns a reference to the input value used for this benchmark run.
    ///
    /// <details>
    /// <summary>Japanese</summary>
    ///
    /// このベンチマーク実行で使用される入力値への参照を返します。
    /// </details>
    pub fn input(&self) -> &I {
        &self.input
    }

    /// Executes the benchmark and returns the measurement results.
    ///
    /// This method performs the following steps:
    /// 1. **Warm-up**: Executes the function several times to warm up the CPU cache.
    /// 2. **Sampling**: Runs the function multiple times to find the minimum CPU cycles (and real time if the `real_time` feature is enabled) to filter out OS noise.
    /// 3. **Allocation Tracking**: Executes the function exactly once while tracking global memory allocations and deallocations.
    ///
    /// <details>
    /// <summary>Japanese</summary>
    ///
    /// ベンチマークを実行し、計測結果を返します。
    ///
    /// このメソッドは以下のステップを実行します：
    /// 1. **ウォームアップ**: CPUキャッシュを温めるために関数を複数回実行します。
    /// 2. **サンプリング**: 関数を複数回実行し、OSのノイズを排除するために最小のCPUサイクル数（`real_time`機能が有効な場合は実時間も）を取得します。
    /// 3. **アロケーション追跡**: グローバルなメモリの割り当てと解放を追跡しながら、関数を正確に1回実行します。
    /// </details>
    #[cfg(target_os = "linux")]
    pub fn run(&mut self) -> Measurement {
        use perf_event::{Builder, Group, events::Hardware};

        for _ in 0..100 {
            std::hint::black_box((self.function)(&self.input));
        }

        let mut group = Group::new().expect("権限エラー: perf_event_paranoid を確認してください");
        let cycles_counter = Builder::new()
            .group(&mut group)
            .kind(Hardware::CPU_CYCLES)
            .exclude_kernel(true)
            .build()
            .unwrap();
        let inst_counter = Builder::new()
            .group(&mut group)
            .kind(Hardware::INSTRUCTIONS)
            .exclude_kernel(true)
            .build()
            .unwrap();

        let samples = 100;
        let mut min_cycles = u64::MAX;
        let mut min_inst = u64::MAX;
        let mut min_time_ns: Option<u64> = None;

        for _ in 0..samples {
            group.reset().unwrap();
            group.enable().unwrap();

            #[cfg(feature = "real_time")]
            let start_time = std::time::Instant::now();

            std::hint::black_box((self.function)(&self.input));

            #[cfg(feature = "real_time")]
            {
                let elapsed = start_time.elapsed().as_nanos() as u64;
                min_time_ns = Some(min_time_ns.unwrap_or(u64::MAX).min(elapsed));
            }

            group.disable().unwrap();
            let counts = group.read().unwrap();

            let c = counts[&cycles_counter];
            let i = counts[&inst_counter];

            if c < min_cycles {
                min_cycles = c;
            }
            if i < min_inst {
                min_inst = i;
            }
        }

        let start_allocs = crate::ALLOC_COUNT.load(Ordering::SeqCst);
        let start_bytes = crate::ALLOC_BYTES.load(Ordering::SeqCst);
        let start_deallocs = crate::DEALLOC_COUNT.load(Ordering::SeqCst);
        let start_dealloc_bytes = crate::DEALLOC_BYTES.load(Ordering::SeqCst);

        std::hint::black_box((self.function)(&self.input));

        let end_allocs = crate::ALLOC_COUNT.load(Ordering::SeqCst);
        let end_bytes = crate::ALLOC_BYTES.load(Ordering::SeqCst);
        let end_deallocs = crate::DEALLOC_COUNT.load(Ordering::SeqCst);
        let end_dealloc_bytes = crate::DEALLOC_BYTES.load(Ordering::SeqCst);

        Measurement::new(
            min_cycles,
            Some(min_inst),
            min_time_ns, // ← 追加: 実時間
            end_allocs - start_allocs,
            end_bytes - start_bytes,
            end_deallocs - start_deallocs,
            end_dealloc_bytes - start_dealloc_bytes,
        )
    }

    /// Executes the benchmark and returns the measurement results.
    ///
    /// This method performs the following steps:
    /// 1. **Warm-up**: Executes the function several times to warm up the CPU cache.
    /// 2. **Sampling**: Runs the function multiple times to find the minimum CPU cycles (and real time if the `real_time` feature is enabled) to filter out OS noise.
    /// 3. **Allocation Tracking**: Executes the function exactly once while tracking global memory allocations and deallocations.
    ///
    /// <details>
    /// <summary>Japanese</summary>
    ///
    /// ベンチマークを実行し、計測結果を返します。
    ///
    /// このメソッドは以下のステップを実行します：
    /// 1. **ウォームアップ**: CPUキャッシュを温めるために関数を複数回実行します。
    /// 2. **サンプリング**: 関数を複数回実行し、OSのノイズを排除するために最小のCPUサイクル数（`real_time`機能が有効な場合は実時間も）を取得します。
    /// 3. **アロケーション追跡**: グローバルなメモリの割り当てと解放を追跡しながら、関数を正確に1回実行します。
    /// </details>
    #[cfg(not(target_os = "linux"))]
    pub fn run(&mut self) -> Measurement {
        for _ in 0..100 {
            std::hint::black_box((self.function)(&self.input));
        }

        let samples = 100;
        let mut min_cycles = u64::MAX;
        let min_time_ns: Option<u64> = None;

        #[cfg(target_arch = "x86_64")]
        {
            use core::arch::x86_64::{__rdtscp, _mm_lfence, _rdtsc};
            for _ in 0..samples {
                unsafe {
                    _mm_lfence();
                    let start = _rdtsc();
                    _mm_lfence();

                    #[cfg(feature = "real_time")]
                    let start_time = std::time::Instant::now();

                    std::hint::black_box((self.function)(&self.input));

                    #[cfg(feature = "real_time")]
                    {
                        let elapsed = start_time.elapsed().as_nanos() as u64;
                        min_time_ns = Some(min_time_ns.unwrap_or(u64::MAX).min(elapsed));
                    }

                    let mut aux: u32 = 0;
                    let end = __rdtscp(&mut aux);
                    _mm_lfence();

                    let elapsed = end - start;
                    if elapsed < min_cycles {
                        min_cycles = elapsed;
                    }
                }
            }
        }

        #[cfg(not(target_arch = "x86_64"))]
        {
            for _ in 0..samples {
                let start = std::time::Instant::now();
                std::hint::black_box((self.function)(&self.input));
                let elapsed = start.elapsed().as_nanos() as u64;
                if elapsed < min_cycles {
                    min_cycles = elapsed;
                }
                #[cfg(feature = "real_time")]
                {
                    min_time_ns = Some(min_time_ns.unwrap_or(u64::MAX).min(elapsed));
                }
            }
        }

        let start_allocs = crate::ALLOC_COUNT.load(Ordering::SeqCst);
        let start_bytes = crate::ALLOC_BYTES.load(Ordering::SeqCst);
        let start_deallocs = crate::DEALLOC_COUNT.load(Ordering::SeqCst);
        let start_dealloc_bytes = crate::DEALLOC_BYTES.load(Ordering::SeqCst);

        std::hint::black_box((self.function)(&self.input));

        let end_allocs = crate::ALLOC_COUNT.load(Ordering::SeqCst);
        let end_bytes = crate::ALLOC_BYTES.load(Ordering::SeqCst);
        let end_deallocs = crate::DEALLOC_COUNT.load(Ordering::SeqCst);
        let end_dealloc_bytes = crate::DEALLOC_BYTES.load(Ordering::SeqCst);

        Measurement::new(
            min_cycles,
            None,
            min_time_ns, // ← 追加: 実時間
            end_allocs - start_allocs,
            end_bytes - start_bytes,
            end_deallocs - start_deallocs,
            end_dealloc_bytes - start_dealloc_bytes,
        )
    }
}
