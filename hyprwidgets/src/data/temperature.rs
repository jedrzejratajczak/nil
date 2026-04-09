use std::fs;
use std::path::PathBuf;

pub struct TempReader {
    cpu_path: PathBuf,
    gpu_path: PathBuf,
    nvme_path: PathBuf,
}

pub struct Temperatures {
    pub cpu: Option<f32>,
    pub gpu: Option<f32>,
    pub nvme: Option<f32>,
}

impl TempReader {
    /// Scan /sys/class/hwmon/ to find sensors by name.
    pub fn discover() -> Option<Self> {
        let mut cpu_path = None;
        let mut gpu_path = None;
        let mut nvme_path = None;

        let entries = fs::read_dir("/sys/class/hwmon").ok()?;
        for entry in entries.flatten() {
            let dir = entry.path();
            let name = fs::read_to_string(dir.join("name"))
                .unwrap_or_default()
                .trim()
                .to_string();

            match name.as_str() {
                "k10temp" => cpu_path = Some(dir.join("temp1_input")),
                "amdgpu" => gpu_path = Some(dir.join("temp1_input")),
                "nvme" => nvme_path = Some(dir.join("temp1_input")),
                _ => {}
            }
        }

        // Require at least CPU sensor
        Some(Self {
            cpu_path: cpu_path?,
            gpu_path: gpu_path.unwrap_or_default(),
            nvme_path: nvme_path.unwrap_or_default(),
        })
    }

    pub fn read(&self) -> Temperatures {
        Temperatures {
            cpu: read_temp(&self.cpu_path),
            gpu: read_temp(&self.gpu_path),
            nvme: read_temp(&self.nvme_path),
        }
    }
}

fn read_temp(path: &PathBuf) -> Option<f32> {
    if path.as_os_str().is_empty() {
        return None;
    }
    let content = fs::read_to_string(path).ok()?;
    let millidegrees: f32 = content.trim().parse().ok()?;
    Some(millidegrees / 1000.0)
}
