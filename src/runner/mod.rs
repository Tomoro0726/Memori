//! Core execution engine for running benchmarks.
//!
//! <details>
//! <summary>Japanese</summary>
//! ベンチマークを実行するためのコアエンジン。
//! </details>

use crate::Measurement;
pub mod measurement;

/// The minimal execution unit for running benchmarks.
///
/// Encapsulates the target function and input, providing high-precision
/// measurements of CPU cycles, execution time, and memory allocations.
///
/// <details>
/// <summary>Japanese</summary>
/// ベンチマーク実行の最小単位。
/// ターゲット関数と入力をカプセル化し、CPUサイクル、実行時間、メモリ割り当てなどを高精度に計測します。
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
    #[cfg(target_os = "linux")]
    pub fn run(&mut self) -> Measurement {
        use perf_event::{Builder, events::Hardware};

        for _ in 0..100 {
            let input = std::hint::black_box(&self.input);
            let _ = std::hint::black_box((self.function)(input));
        }

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
        let mut min_fallback_cycles = u64::MAX;
        let mut min_inst = u64::MAX;

        #[cfg(feature = "real_time")]
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
            let start_fallback = unsafe {
                core::arch::x86_64::_mm_lfence();
                let c = core::arch::x86_64::_rdtsc();
                core::arch::x86_64::_mm_lfence();
                c
            };

            #[cfg(target_arch = "aarch64")]
            let start_fallback = {
                let mut c: u64;
                unsafe {
                    core::arch::asm!("isb", "mrs {}, cntvct_el0", out(reg) c, options(nomem, nostack));
                }
                c
            };

            #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
            let start_fallback = 0;

            #[cfg(feature = "real_time")]
            let start_time = std::time::Instant::now();

            let input = std::hint::black_box(&self.input);
            let _ = std::hint::black_box((self.function)(input));

            #[cfg(feature = "real_time")]
            {
                let elapsed = start_time.elapsed().as_nanos() as u64;
                min_time_ns = Some(min_time_ns.map_or(elapsed, |prev| prev.min(elapsed)));
            }

            #[cfg(target_arch = "x86_64")]
            {
                let mut aux = 0;
                let end_fallback = unsafe { core::arch::x86_64::__rdtscp(&mut aux) };
                unsafe { core::arch::x86_64::_mm_lfence() };
                min_fallback_cycles =
                    min_fallback_cycles.min(end_fallback.wrapping_sub(start_fallback));
            }

            #[cfg(target_arch = "aarch64")]
            {
                let mut end_fallback: u64;
                unsafe {
                    core::arch::asm!("isb", "mrs {}, cntvct_el0", out(reg) end_fallback, options(nomem, nostack));
                }
                min_fallback_cycles =
                    min_fallback_cycles.min(end_fallback.wrapping_sub(start_fallback));
            }

            #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
            {
                min_fallback_cycles = 0;
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

        let s_alloc = crate::allocator::THREAD_ALLOC_COUNT
            .try_with(|c| c.get())
            .unwrap_or(0);
        let s_bytes = crate::allocator::THREAD_ALLOC_BYTES
            .try_with(|c| c.get())
            .unwrap_or(0);
        let s_dealloc = crate::allocator::THREAD_DEALLOC_COUNT
            .try_with(|c| c.get())
            .unwrap_or(0);
        let s_dealloc_bytes = crate::allocator::THREAD_DEALLOC_BYTES
            .try_with(|c| c.get())
            .unwrap_or(0);

        let input = std::hint::black_box(&self.input);
        let _ = std::hint::black_box((self.function)(input));

        let e_alloc = crate::allocator::THREAD_ALLOC_COUNT
            .try_with(|c| c.get())
            .unwrap_or(0);
        let e_bytes = crate::allocator::THREAD_ALLOC_BYTES
            .try_with(|c| c.get())
            .unwrap_or(0);
        let e_dealloc = crate::allocator::THREAD_DEALLOC_COUNT
            .try_with(|c| c.get())
            .unwrap_or(0);
        let e_dealloc_bytes = crate::allocator::THREAD_DEALLOC_BYTES
            .try_with(|c| c.get())
            .unwrap_or(0);

        let final_cycles = if min_perf_cycles != u64::MAX {
            min_perf_cycles
        } else if min_fallback_cycles != u64::MAX {
            min_fallback_cycles
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
            min_time_ns,
            #[cfg(not(feature = "real_time"))]
            None,
            e_alloc.wrapping_sub(s_alloc),
            e_bytes.wrapping_sub(s_bytes),
            e_dealloc.wrapping_sub(s_dealloc),
            e_dealloc_bytes.wrapping_sub(s_dealloc_bytes),
        )
    }

    /// Executes the benchmark and returns the measurement results.
    #[cfg(not(target_os = "linux"))]
    pub fn run(&mut self) -> Measurement {
        for _ in 0..100 {
            let input = std::hint::black_box(&self.input);
            let _ = std::hint::black_box((self.function)(input));
        }

        let samples = 100;
        let mut min_cycles = u64::MAX;

        #[cfg(feature = "real_time")]
        let mut min_time_ns: Option<u64> = None;

        for _ in 0..samples {
            #[cfg(target_arch = "x86_64")]
            let start_cycles = unsafe {
                core::arch::x86_64::_mm_lfence();
                let c = core::arch::x86_64::_rdtsc();
                core::arch::x86_64::_mm_lfence();
                c
            };

            #[cfg(target_arch = "aarch64")]
            let start_cycles = {
                let mut c: u64;
                unsafe {
                    core::arch::asm!("isb", "mrs {}, cntvct_el0", out(reg) c, options(nomem, nostack));
                }
                c
            };

            #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
            let start_cycles = 0;

            #[cfg(feature = "real_time")]
            let start_time = std::time::Instant::now();

            let input = std::hint::black_box(&self.input);
            let _ = std::hint::black_box((self.function)(input));

            #[cfg(target_arch = "x86_64")]
            {
                let mut aux = 0;
                let end_cycles = unsafe { core::arch::x86_64::__rdtscp(&mut aux) };
                unsafe { core::arch::x86_64::_mm_lfence() };
                min_cycles = min_cycles.min(end_cycles.wrapping_sub(start_cycles));
            }

            #[cfg(target_arch = "aarch64")]
            {
                let mut end_cycles: u64;
                unsafe {
                    core::arch::asm!("isb", "mrs {}, cntvct_el0", out(reg) end_cycles, options(nomem, nostack));
                }
                min_cycles = min_cycles.min(end_cycles.wrapping_sub(start_cycles));
            }

            #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
            {
                min_cycles = 0;
            }

            #[cfg(feature = "real_time")]
            {
                let elapsed = start_time.elapsed().as_nanos() as u64;
                min_time_ns = Some(min_time_ns.map_or(elapsed, |prev| prev.min(elapsed)));
            }
        }

        let s_alloc = crate::allocator::THREAD_ALLOC_COUNT
            .try_with(|c| c.get())
            .unwrap_or(0);
        let s_bytes = crate::allocator::THREAD_ALLOC_BYTES
            .try_with(|c| c.get())
            .unwrap_or(0);
        let s_dealloc = crate::allocator::THREAD_DEALLOC_COUNT
            .try_with(|c| c.get())
            .unwrap_or(0);
        let s_dealloc_bytes = crate::allocator::THREAD_DEALLOC_BYTES
            .try_with(|c| c.get())
            .unwrap_or(0);

        let input = std::hint::black_box(&self.input);
        let _ = std::hint::black_box((self.function)(input));

        let e_alloc = crate::allocator::THREAD_ALLOC_COUNT
            .try_with(|c| c.get())
            .unwrap_or(0);
        let e_bytes = crate::allocator::THREAD_ALLOC_BYTES
            .try_with(|c| c.get())
            .unwrap_or(0);
        let e_dealloc = crate::allocator::THREAD_DEALLOC_COUNT
            .try_with(|c| c.get())
            .unwrap_or(0);
        let e_dealloc_bytes = crate::allocator::THREAD_DEALLOC_BYTES
            .try_with(|c| c.get())
            .unwrap_or(0);

        Measurement::new(
            if min_cycles == u64::MAX {
                0
            } else {
                min_cycles
            },
            None,
            #[cfg(feature = "real_time")]
            min_time_ns,
            #[cfg(not(feature = "real_time"))]
            None,
            e_alloc.wrapping_sub(s_alloc),
            e_bytes.wrapping_sub(s_bytes),
            e_dealloc.wrapping_sub(s_dealloc),
            e_dealloc_bytes.wrapping_sub(s_dealloc_bytes),
        )
    }
}
