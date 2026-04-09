use std::fs;

pub struct MemInfo {
    pub used_gb: f32,
    pub total_gb: f32,
    pub percent: f32,
}

pub fn read_memory() -> MemInfo {
    let content = fs::read_to_string("/proc/meminfo").unwrap_or_default();
    let mut total_kb: u64 = 0;
    let mut available_kb: u64 = 0;

    for line in content.lines() {
        if let Some(val) = line.strip_prefix("MemTotal:") {
            total_kb = parse_kb(val);
        } else if let Some(val) = line.strip_prefix("MemAvailable:") {
            available_kb = parse_kb(val);
        }
        if total_kb > 0 && available_kb > 0 {
            break;
        }
    }

    let total_gb = total_kb as f32 / 1_048_576.0;
    let used_gb = (total_kb - available_kb) as f32 / 1_048_576.0;
    let percent = if total_kb > 0 {
        used_gb / total_gb * 100.0
    } else {
        0.0
    };

    MemInfo {
        used_gb,
        total_gb,
        percent,
    }
}

fn parse_kb(s: &str) -> u64 {
    s.split_whitespace()
        .next()
        .and_then(|v| v.parse().ok())
        .unwrap_or(0)
}
