mod app;
mod thumbnail;
mod wallpaper;

use std::os::unix::net::UnixStream;

pub fn socket_path() -> String {
    format!("/tmp/hyprwall-{}.sock", unsafe { libc::getuid() })
}

fn main() {
    // If daemon is already running, send toggle and exit
    if UnixStream::connect(socket_path()).is_ok() {
        return;
    }

    // No daemon running — become the daemon
    let lock_path = format!("/tmp/hyprwall-{}.lock", unsafe { libc::getuid() });
    let lock_file = std::fs::File::create(&lock_path).expect("cannot create lock file");
    use std::os::unix::io::AsRawFd;
    let ret = unsafe { libc::flock(lock_file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) };
    if ret != 0 {
        eprintln!("hyprwall daemon is already running");
        std::process::exit(1);
    }
    std::mem::forget(lock_file);

    app::run();
}
