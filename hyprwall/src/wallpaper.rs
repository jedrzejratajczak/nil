use std::path::PathBuf;

pub struct WallpaperEntry {
    pub path: PathBuf,
    pub is_gif: bool,
}

const SUPPORTED_EXTENSIONS: &[&str] = &["jpg", "jpeg", "png", "gif", "webp", "bmp"];

pub fn discover_wallpapers() -> Vec<WallpaperEntry> {
    let dir = dirs::home_dir()
        .expect("no home dir")
        .join("Pictures/Wallpapers");

    let Ok(entries) = std::fs::read_dir(&dir) else {
        eprintln!("cannot read {}", dir.display());
        return Vec::new();
    };

    let mut wallpapers: Vec<WallpaperEntry> = entries
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().ok().is_some_and(|ft| ft.is_file()))
        .filter_map(|e| {
            let path = e.path();
            let ext = path.extension()?.to_str()?.to_ascii_lowercase();
            SUPPORTED_EXTENSIONS.contains(&ext.as_str()).then(|| WallpaperEntry {
                is_gif: ext == "gif",
                path,
            })
        })
        .collect();

    wallpapers.sort_by(|a, b| a.path.file_name().cmp(&b.path.file_name()));
    wallpapers
}
