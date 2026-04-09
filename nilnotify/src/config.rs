use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct Colors {
    pub bg: String,
    pub fg: String,
    pub accent: String,
    pub surface: String,
    pub outline: String,
}

impl Default for Colors {
    fn default() -> Self {
        Self {
            bg: "#1a1a1a".into(),
            fg: "#e0e0e0".into(),
            accent: "#87d6bd".into(),
            surface: "#2a2a2a".into(),
            outline: "#444444".into(),
        }
    }
}

impl Colors {
    pub fn load() -> Self {
        let path = colors_path();
        let content = match fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => {
                eprintln!("nilnotify: could not read {}, using defaults", path.display());
                return Self::default();
            }
        };

        let map: HashMap<&str, &str> = content
            .lines()
            .filter_map(|line| {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') {
                    return None;
                }
                let (key, val) = line.split_once('=')?;
                Some((key.trim(), val.trim()))
            })
            .collect();

        Self {
            bg: map.get("bg").unwrap_or(&"#1a1a1a").to_string(),
            fg: map.get("fg").unwrap_or(&"#e0e0e0").to_string(),
            accent: map.get("accent").unwrap_or(&"#87d6bd").to_string(),
            surface: map.get("surface").unwrap_or(&"#2a2a2a").to_string(),
            outline: map.get("outline").unwrap_or(&"#444444").to_string(),
        }
    }
}

fn colors_path() -> PathBuf {
    let config = std::env::var("XDG_CONFIG_HOME")
        .unwrap_or_else(|_| format!("{}/.config", std::env::var("HOME").unwrap()));
    PathBuf::from(config).join("nilnotify/colors")
}
