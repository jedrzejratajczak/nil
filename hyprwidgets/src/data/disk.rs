use std::mem::MaybeUninit;

pub struct DiskInfo {
    pub used_gb: f32,
    pub total_gb: f32,
    pub percent: f32,
}

pub fn read_disk() -> DiskInfo {
    let mut buf = MaybeUninit::<libc::statvfs>::uninit();
    let ret = unsafe { libc::statvfs(c"/".as_ptr(), buf.as_mut_ptr()) };
    if ret != 0 {
        return DiskInfo {
            used_gb: 0.0,
            total_gb: 0.0,
            percent: 0.0,
        };
    }

    let stat = unsafe { buf.assume_init() };
    let block_size = stat.f_frsize as u64;
    let total = stat.f_blocks as u64 * block_size;
    let available = stat.f_bavail as u64 * block_size;
    let used = total - (stat.f_bfree as u64 * block_size);

    let total_gb = total as f32 / (1024.0 * 1024.0 * 1024.0);
    let used_gb = used as f32 / (1024.0 * 1024.0 * 1024.0);
    let percent = if total > 0 {
        used as f32 / (used + available) as f32 * 100.0
    } else {
        0.0
    };

    DiskInfo {
        used_gb,
        total_gb,
        percent,
    }
}
