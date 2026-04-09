use std::fs;

pub struct CpuSampler {
    prev_idle: u64,
    prev_total: u64,
}

impl CpuSampler {
    pub fn new() -> Self {
        let (idle, total) = read_cpu_times();
        Self {
            prev_idle: idle,
            prev_total: total,
        }
    }

    pub fn sample(&mut self) -> f32 {
        let (idle, total) = read_cpu_times();
        let d_idle = idle.saturating_sub(self.prev_idle);
        let d_total = total.saturating_sub(self.prev_total);
        self.prev_idle = idle;
        self.prev_total = total;

        if d_total == 0 {
            return 0.0;
        }
        (1.0 - d_idle as f64 / d_total as f64) as f32 * 100.0
    }
}

fn read_cpu_times() -> (u64, u64) {
    let content = fs::read_to_string("/proc/stat").unwrap_or_default();
    let line = match content.lines().next() {
        Some(l) if l.starts_with("cpu ") => l,
        _ => return (0, 0),
    };

    let values: Vec<u64> = line
        .split_whitespace()
        .skip(1) // skip "cpu"
        .filter_map(|s| s.parse().ok())
        .collect();

    if values.len() < 4 {
        return (0, 0);
    }

    // user, nice, system, idle, iowait, irq, softirq, steal
    let idle = values[3] + values.get(4).copied().unwrap_or(0);
    let total: u64 = values.iter().sum();
    (idle, total)
}
