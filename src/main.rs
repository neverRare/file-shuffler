use clap::Parser;
use rand::{thread_rng, Rng};
use std::{
    env::current_dir,
    ffi::OsString,
    fs::{read_dir, rename},
    io::{self, stdout, Write},
    path::PathBuf,
};

const MAX: u128 = 170581728179578208256; // 36 ^ 13

/// Simple utility for randomizing file names
#[derive(Parser, Debug)]
struct Args {
    /// The directory to randomize file names
    path: PathBuf,
}
fn main() -> io::Result<()> {
    let path = Args::parse().path;
    let full_path = current_dir()?.join(path);
    let paths: Vec<_> = read_dir(&full_path)?
        .filter_map(Result::ok)
        .map(|dir| dir.path())
        .collect();
    let len = paths.len();
    println!("found {len} files");
    let mut rng = thread_rng();
    for (i, file) in paths.into_iter().enumerate() {
        print!("\rrenaming {}/{len}", i);
        stdout().flush()?;
        let extension = file.extension();
        let new_path = loop {
            let mut name = base36(rng.gen_range(0..MAX));
            if let Some(extension) = extension {
                name.push(".");
                name.push(extension);
            }
            let path = full_path.join(name);
            if !path.try_exists()? {
                break path;
            }
        };
        rename(file, new_path)?;
    }
    println!();
    Ok(())
}
fn base36(mut x: u128) -> OsString {
    let mut result = Vec::with_capacity(13);
    loop {
        let m = (x % 36) as u8;
        x = x / 36;
        let byte = if m < 10 { b'0' + m } else { b'a' + m - 10 };
        result.push(byte);
        if x == 0 {
            break;
        }
    }
    while result.len() < 13 {
        result.push(b'0');
    }
    result.reverse();
    let string = unsafe {String::from_utf8_unchecked(result)};
    string.into()
}
