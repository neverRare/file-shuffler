use clap::Parser;
use rand::{thread_rng, Rng};
use std::{
    env::current_dir,
    ffi::{OsStr, OsString},
    fs::{read_dir, rename},
    io::{self, stdout, Write},
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
    thread::{sleep, spawn},
    time::Duration,
};

const MAX: u128 = 170581728179578208256; // 36 ^ 13
static PROGRESS: AtomicUsize = AtomicUsize::new(0);

/// Simple utility for randomizing file names
#[derive(Parser, Debug)]
struct Args {
    /// The directory to randomize file names
    path: PathBuf,
}
fn main() -> io::Result<()> {
    let path = Args::parse().path;
    let target = current_dir()?.join(path);
    // TODO: use try_collect if stable
    let mut paths = Vec::new();
    for entry in read_dir(&target)? {
        paths.push(entry?.path());
    }
    let len = paths.len();
    println!("found {len} files");
    let mut rng = thread_rng();
    spawn(move || {
        sleep(Duration::from_secs(1));
        print!(
            "\rRenamed {} / {len} files",
            PROGRESS.load(Ordering::Relaxed)
        );
        stdout().flush().unwrap();
    });
    for (i, path) in paths.into_iter().enumerate() {
        let extension = path.extension();
        let extension_len = extension
            .map(OsStr::len)
            .map(|len| len + 1)
            .unwrap_or_default();
        let new_path = loop {
            let mut name = base36(rng.gen_range(0..MAX), extension_len);
            if let Some(extension) = extension {
                name.push(".");
                name.push(extension);
            }
            let path = target.join(name);
            if !path.try_exists()? {
                break path;
            }
        };
        rename(path, new_path)?;
        PROGRESS.store(i + 1, Ordering::Relaxed);
    }
    println!("\rRenamed {len} / {len} files");
    Ok(())
}
fn base36(mut x: u128, extension_len: usize) -> OsString {
    let mut result = Vec::with_capacity(13 + extension_len);
    result.extend([0; 13]);
    for i in 0..13 {
        let m = (x % 36) as u8;
        x = x / 36;
        let byte = if m < 10 { b'0' + m } else { b'a' + m - 10 };
        result[13 - i - 1] = byte;
    }
    let string = unsafe { String::from_utf8_unchecked(result) };
    string.into()
}
