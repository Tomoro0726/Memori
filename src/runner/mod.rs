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
        // Groupを削除し、独立して扱う
        use perf_event::{Builder, events::Hardware};

        // ウォームアップ
        for _ in 0..100 {
            std::hint::black_box((self.function)(&self.input));
        }

        let mut cycles_counter = {
            // Builder::new() と .kind() は所有権を返すので繋げてOK
            let mut b = Builder::new().kind(Hardware::CPU_CYCLES);
            // exclude_kernel は &mut (参照) を返すので単独の行で呼ぶ
            b.exclude_kernel(true);
            // 変数 b 本体から build() を呼んで所有権を消費する
            b.build().ok()
        };

        let mut inst_counter = {
            let mut b = Builder::new().kind(Hardware::INSTRUCTIONS);
            b.exclude_kernel(true);
            b.build().ok()
        };

        let samples = 100;
        let mut min_cycles = u64::MAX;
        let mut min_inst = u64::MAX;
        let mut min_time_ns: Option<u64> = None;

        for _ in 0..samples {
            // それぞれ独立してリセット＆開始
            if let Some(c) = cycles_counter.as_mut() {
                let _ = c.reset();
                let _ = c.enable();
            }
            if let Some(i) = inst_counter.as_mut() {
                let _ = i.reset();
                let _ = i.enable();
            }

            #[cfg(feature = "real_time")]
            let start_time = std::time::Instant::now();

            std::hint::black_box((self.function)(&self.input));

            #[cfg(feature = "real_time")]
            {
                let elapsed = start_time.elapsed().as_nanos() as u64;
                match min_time_ns {
                    Some(prev) => {
                        if elapsed < prev {
                            min_time_ns = Some(elapsed);
                        }
                    }
                    None => {
                        min_time_ns = Some(elapsed);
                    }
                }
            }

            // それぞれ独立して停止＆読み取り
            if let Some(i) = inst_counter.as_mut() {
                let _ = i.disable();
                if let Ok(count) = i.read() {
                    if count < min_inst {
                        min_inst = count;
                    }
                }
            }
            if let Some(c) = cycles_counter.as_mut() {
                let _ = c.disable();
                if let Ok(count) = c.read() {
                    if count < min_cycles {
                        min_cycles = count;
                    }
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

        // フォールバック処理
        let final_cycles = if min_cycles == u64::MAX {
            0
        } else {
            min_cycles
        };
        let final_inst = if min_inst == u64::MAX {
            None
        } else {
            Some(min_inst)
        };

        Measurement::new(
            final_cycles,
            final_inst,
            min_time_ns.or(Some(0)),
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
        let mut min_time_ns: Option<u64> = None;

        #[cfg(target_arch = "x86_64")]
        {
            use core::arch::x86_64::{__rdtscp, _mm_lfence, _rdtsc};
            for _ in 0..samples {
                unsafe {
                    _mm_lfence();
                    let start_cycles = _rdtsc();
                    _mm_lfence();

                    let start_time = std::time::Instant::now();
                    std::hint::black_box((self.function)(&self.input));
                    let elapsed_time = start_time.elapsed().as_nanos() as u64;

                    // timeNs: always measure
                    match min_time_ns {
                        Some(prev) => {
                            if elapsed_time < prev {
                                min_time_ns = Some(elapsed_time);
                            }
                        }
                        None => {
                            min_time_ns = Some(elapsed_time);
                        }
                    }

                    let mut aux: u32 = 0;
                    let end_cycles = __rdtscp(&mut aux);
                    _mm_lfence();

                    let elapsed_cycles = end_cycles - start_cycles;
                    if elapsed_cycles < min_cycles {
                        min_cycles = elapsed_cycles;
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
                    match min_time_ns {
                        Some(prev) => {
                            if elapsed < prev {
                                min_time_ns = Some(elapsed);
                            }
                        }
                        None => {
                            min_time_ns = Some(elapsed);
                        }
                    }
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
            min_time_ns.or(Some(0)), // ← 実時間が取得できない場合は0を記録
            end_allocs - start_allocs,
            end_bytes - start_bytes,
            end_deallocs - start_deallocs,
            end_dealloc_bytes - start_dealloc_bytes,
        )
    }
}
