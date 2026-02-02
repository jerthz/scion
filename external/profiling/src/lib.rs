use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::collections::HashMap;

static PROFILING_ENABLED: AtomicBool = AtomicBool::new(true);
static PROFILE_DATA: Mutex<Option<HashMap<&'static str, Vec<u128>>>> = Mutex::new(None);

pub struct ProfileGuard {
    name: &'static str,
    start: std::time::Instant,
}

impl ProfileGuard {
    #[inline(always)]
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            start: std::time::Instant::now(),
        }
    }
}

impl Drop for ProfileGuard {
    #[inline(always)]
    fn drop(&mut self) {
        if !PROFILING_ENABLED.load(Ordering::Relaxed) {
            return;
        }

        let elapsed = self.start.elapsed().as_nanos();

        // Store data instead of printing immediately
        if let Ok(mut data) = PROFILE_DATA.lock() {
            if data.is_none() {
                *data = Some(HashMap::new());
            }
            if let Some(map) = data.as_mut() {
                map.entry(self.name).or_insert_with(Vec::new).push(elapsed);
            }
        }
    }
}

/// Print accumulated profiling data
pub fn print_profile_stats() {
    if let Ok(mut data) = PROFILE_DATA.lock() {
        if let Some(map) = data.take() {
            for (name, times) in map.iter() {
                let count = times.len();
                let total: u128 = times.iter().sum();
                let avg = total / count as u128;
                let min = times.iter().min().unwrap();
                let max = times.iter().max().unwrap();

                eprintln!(
                    "[profile] {} | calls: {} | avg: {}ns | min: {}ns | max: {}ns | total: {}µs",
                    name, count, avg, min, max, total / 1000
                );
            }
        }
    }
}

/// Enable/disable profiling at runtime
pub fn set_profiling_enabled(enabled: bool) {
    PROFILING_ENABLED.store(enabled, Ordering::Relaxed);
}

/// Clear accumulated profiling data
pub fn clear_profile_data() {
    if let Ok(mut data) = PROFILE_DATA.lock() {
        *data = None;
    }
}
