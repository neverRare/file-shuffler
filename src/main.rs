use clap::Parser;
use human_panic::setup_panic;
use rand::{thread_rng, Rng};
use std::{
    env::current_dir,
    ffi::OsString,
    fs::{read_dir, rename},
    io::{stdout, Write},
    path::PathBuf,
    process::ExitCode,
    sync::atomic::{AtomicUsize, Ordering},
    thread::{sleep, Builder},
    time::Duration,
};

const MAX: u128 = 170581728179578208256; // 36 ^ 13
static PROGRESS: AtomicUsize = AtomicUsize::new(0);

/// Simple utility for randomizing file names in order to shuffle the sort
/// order
#[derive(Parser, Debug)]
struct Args {
    /// The directory to randomize file names, must point to a directory and
    /// not a file
    path: PathBuf,
}
fn main() -> ExitCode {
    setup_panic!();
    let path = Args::parse().path;
    let target = current_dir().unwrap().join(path);
    let entries = match read_dir(&target) {
        Ok(entries) => entries,
        Err(err) => {
            eprintln!("unable to read {}: {}", target.display(), err);
            return ExitCode::FAILURE;
        }
    };
    // TODO: use try_collect if stable
    let mut paths = Vec::new();
    for entry in entries {
        paths.push(entry.unwrap().path());
    }
    let len = paths.len();
    println!("found {len} files");
    let mut rng = thread_rng();
    let tracked = Builder::new()
        .spawn(move || loop {
            sleep(Duration::from_secs(1));
            print!(
                "\rRenamed {} / {len} files",
                PROGRESS.load(Ordering::Relaxed)
            );
            stdout().flush().unwrap();
        })
        .is_ok();
    for (i, path) in paths.into_iter().enumerate() {
        let extension = path.extension();
        let new_path = loop {
            let mut name = base36(rng.gen_range(0..MAX));
            if let Some(extension) = extension {
                name.push(".");
                name.push(extension);
            }
            let path = target.join(name);
            if !path.try_exists().unwrap() {
                break path;
            }
        };
        rename(path, new_path).unwrap();
        if tracked {
            PROGRESS.store(i + 1, Ordering::Relaxed);
        } else {
            print!("\rRenamed {} / {len} files", i + 1);
            stdout().flush().unwrap();
        }
    }
    println!("\rRenamed {len} / {len} files");
    ExitCode::SUCCESS
}
fn base36(mut x: u128) -> OsString {
    let mut result = vec![0; 13];
    for i in 0..13 {
        let m = (x % 36) as u8;
        x /= 36;
        let byte = if m < 10 { b'0' + m } else { b'a' + m - 10 };
        result[13 - i - 1] = byte;
    }
    let string = unsafe { String::from_utf8_unchecked(result) };
    string.into()
}
