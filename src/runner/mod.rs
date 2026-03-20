use crate::Measurement;
pub mod measurement;

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
        Self { input, function }
    }

    pub fn input(&self) -> &I {
        &self.input
    }

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
        let mut min_rdtsc_cycles = u64::MAX;
        let mut min_inst = u64::MAX;
        let mut min_time_ns = u64::MAX;

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

            let start_time = std::time::Instant::now();

            let input = std::hint::black_box(&self.input);
            let _ = std::hint::black_box((self.function)(input));

            let elapsed = start_time.elapsed().as_nanos() as u64;
            min_time_ns = min_time_ns.min(elapsed);

            #[cfg(target_arch = "x86_64")]
            {
                let mut aux = 0;
                let end_rdtsc = unsafe { core::arch::x86_64::__rdtscp(&mut aux) };
                unsafe { core::arch::x86_64::_mm_lfence() };
                min_rdtsc_cycles = min_rdtsc_cycles.min(end_rdtsc.wrapping_sub(start_rdtsc));
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
            Some(min_time_ns),
            e_alloc.wrapping_sub(s_alloc),
            e_bytes.wrapping_sub(s_bytes),
            e_dealloc.wrapping_sub(s_dealloc),
            e_dealloc_bytes.wrapping_sub(s_dealloc_bytes),
        )
    }

    #[cfg(not(target_os = "linux"))]
    pub fn run(&mut self) -> Measurement {
        for _ in 0..100 {
            let input = std::hint::black_box(&self.input);
            let _ = std::hint::black_box((self.function)(input));
        }

        let samples = 100;
        let mut min_cycles = u64::MAX;
        let mut min_time_ns = u64::MAX;

        for _ in 0..samples {
            #[cfg(target_arch = "x86_64")]
            let start_cycles = unsafe {
                core::arch::x86_64::_mm_lfence();
                let c = core::arch::x86_64::_rdtsc();
                core::arch::x86_64::_mm_lfence();
                c
            };

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

            let elapsed = start_time.elapsed().as_nanos() as u64;
            min_time_ns = min_time_ns.min(elapsed);
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
            Some(min_time_ns),
            e_alloc.wrapping_sub(s_alloc),
            e_bytes.wrapping_sub(s_bytes),
            e_dealloc.wrapping_sub(s_dealloc),
            e_dealloc_bytes.wrapping_sub(s_dealloc_bytes),
        )
    }
}
