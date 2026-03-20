use crate::Measurement;
pub mod measurement;
use std::sync::atomic::Ordering;

/// The minimal execution unit for running benchmarks.
///
/// Encapsulates the target function and input, providing high-precision
/// measurements of CPU cycles, execution time, and memory allocations.
///
/// <details>
/// <summary>Japanese</summary>
/// ベンチマーク実行の最小単位です。
/// ターゲット関数と入力をカプセル化し、CPUサイクル、実行時間、メモリ割り当てを高精度に計測します。
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
    pub fn new(input: I, function: F) -> Self {
        Self { input, function }
    }

    /// Returns a reference to the input value.
    pub fn input(&self) -> &I {
        &self.input
    }

    /// Executes the benchmark and returns the measurement results.
    ///
    /// 1. **Warm-up**: Stabilizes CPU cache.
    /// 2. **Sampling**: Runs multiple iterations to find the minimum values (filtering noise).
    /// 3. **Allocation Tracking**: Tracks memory via global hooks during a single execution.
    #[cfg(target_os = "linux")]
    pub fn run(&mut self) -> Measurement {
        use perf_event::{Builder, events::Hardware};

        for _ in 0..100 {
            std::hint::black_box((self.function)(&self.input));
        }

        // Use independent counters to avoid data size mismatch panics in VM environments.
        let mut cycles_counter = {
            let mut b = Builder::new().kind(Hardware::CPU_CYCLES);
            b.exclude_kernel(true);
            b.build().ok()
        };

        let mut inst_counter = {
            let mut b = Builder::new().kind(Hardware::INSTRUCTIONS);
            b.exclude_kernel(true);
            b.build().ok()
        };

        let samples = 100;
        let mut min_perf_cycles = u64::MAX;
        let mut min_rdtsc_cycles = u64::MAX;
        let mut min_inst = u64::MAX;
        let mut min_time_ns: Option<u64> = None;

        for _ in 0..samples {
            if let Some(c) = cycles_counter.as_mut() {
                let _ = c.reset();
                let _ = c.enable();
            }
            if let Some(i) = inst_counter.as_mut() {
                let _ = i.reset();
                let _ = i.enable();
            }

            #[cfg(target_arch = "x86_64")]
            let start_rdtsc = unsafe {
                core::arch::x86_64::_mm_lfence();
                let c = core::arch::x86_64::_rdtsc();
                core::arch::x86_64::_mm_lfence();
                c
            };

            #[cfg(feature = "real_time")]
            let start_time = std::time::Instant::now();

            std::hint::black_box((self.function)(&self.input));

            #[cfg(feature = "real_time")]
            {
                let elapsed = start_time.elapsed().as_nanos() as u64;
                min_time_ns = Some(min_time_ns.map_or(elapsed, |prev| prev.min(elapsed)));
            }

            #[cfg(target_arch = "x86_64")]
            {
                let mut aux = 0;
                let end_rdtsc = unsafe { core::arch::x86_64::__rdtscp(&mut aux) };
                unsafe { core::arch::x86_64::_mm_lfence() };
                min_rdtsc_cycles = min_rdtsc_cycles.min(end_rdtsc - start_rdtsc);
            }

            if let Some(i) = inst_counter.as_mut() {
                let _ = i.disable();
                if let Ok(count) = i.read() {
                    min_inst = min_inst.min(count);
                }
            }
            if let Some(c) = cycles_counter.as_mut() {
                let _ = c.disable();
                if let Ok(count) = c.read() {
                    min_perf_cycles = min_perf_cycles.min(count);
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

        let final_cycles = if min_perf_cycles != u64::MAX {
            min_perf_cycles
        } else if min_rdtsc_cycles != u64::MAX {
            min_rdtsc_cycles
        } else {
            0
        };

        Measurement::new(
            final_cycles,
            if min_inst == u64::MAX {
                None
            } else {
                Some(min_inst)
            },
            #[cfg(feature = "real_time")]
            min_time_ns.or(Some(0)),
            #[cfg(not(feature = "real_time"))]
            None,
            end_allocs - start_allocs,
            end_bytes - start_bytes,
            end_deallocs - start_deallocs,
            end_dealloc_bytes - start_dealloc_bytes,
        )
    }

    #[cfg(not(target_os = "linux"))]
    pub fn run(&mut self) -> Measurement {
        for _ in 0..100 {
            std::hint::black_box((self.function)(&self.input));
        }

        let samples = 100;
        let mut min_cycles = u64::MAX;
        let mut min_time_ns: Option<u64> = None;

        for _ in 0..samples {
            #[cfg(target_arch = "x86_64")]
            let start_cycles = unsafe {
                core::arch::x86_64::_mm_lfence();
                let c = core::arch::x86_64::_rdtsc();
                core::arch::x86_64::_mm_lfence();
                c
            };

            #[cfg(feature = "real_time")]
            let start_time = std::time::Instant::now();

            std::hint::black_box((self.function)(&self.input));

            #[cfg(target_arch = "x86_64")]
            {
                // RDTSC timing (after function execution)
                let mut aux = 0;
                let end_cycles = unsafe { core::arch::x86_64::__rdtscp(&mut aux) };
                unsafe { core::arch::x86_64::_mm_lfence() };
                min_cycles = min_cycles.min(end_cycles - start_cycles);
            }

            #[cfg(feature = "real_time")]
            {
                let elapsed = start_time.elapsed().as_nanos() as u64;
                min_time_ns = Some(min_time_ns.map_or(elapsed, |prev| prev.min(elapsed)));
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
            if min_cycles == u64::MAX {
                0
            } else {
                min_cycles
            },
            None,
            #[cfg(feature = "real_time")]
            min_time_ns.or(Some(0)),
            #[cfg(not(feature = "real_time"))]
            None,
            end_allocs - start_allocs,
            end_bytes - start_bytes,
            end_deallocs - start_deallocs,
            end_dealloc_bytes - start_dealloc_bytes,
        )
    }
}
