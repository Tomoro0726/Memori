use crate::Measurement;
pub mod measurement;
use std::sync::atomic::Ordering;

/// ベンチマークの実行の最小単位
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
    pub fn new(input: I, function: F) -> Self {
        Runner { input, function }
    }

    pub fn input(&self) -> &I {
        &self.input
    }

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

        for _ in 0..samples {
            group.reset().unwrap();
            group.enable().unwrap();

            std::hint::black_box((self.function)(&self.input));

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

        // ▼ 追加: 実行後の Dealloc カウンターを取得
        let end_allocs = crate::ALLOC_COUNT.load(Ordering::SeqCst);
        let end_bytes = crate::ALLOC_BYTES.load(Ordering::SeqCst);
        let end_deallocs = crate::DEALLOC_COUNT.load(Ordering::SeqCst);
        let end_dealloc_bytes = crate::DEALLOC_BYTES.load(Ordering::SeqCst);

        Measurement::new(
            min_cycles,
            Some(min_inst),
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

        #[cfg(target_arch = "x86_64")]
        {
            use core::arch::x86_64::{__rdtscp, _mm_lfence, _rdtsc};
            for _ in 0..samples {
                unsafe {
                    _mm_lfence();
                    let start = _rdtsc();
                    _mm_lfence();

                    std::hint::black_box((self.function)(&self.input));

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
            end_allocs - start_allocs,
            end_bytes - start_bytes,
            end_deallocs - start_deallocs,
            end_dealloc_bytes - start_dealloc_bytes,
        )
    }
}
