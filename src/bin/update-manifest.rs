// Chatpack manifest updater in rust, by Blake Oliver <oliver22213@me.com>

// This program hashes files under TARGET_DIR and outputs the result to a file;
// That file is for use by the actual updater

use std::io::{stdout, stderr};
use std::path::PathBuf;
use std::env;
use std::collections::{BTreeSet, BTreeMap};
// pull in checksums
extern crate checksums;
use checksums::ops::create_hashes;
use checksums::Algorithm;

// Pull in chrono
extern crate chrono;
use chrono::{Local, DateTime, Datelike};

extern crate chatpack_updater;
use chatpack_updater::version::Version;

// set constants
const TARGET_DIR: &str = "chatpack"; // The directory (under the current working one) in which files will be hashed (recursing into subdirectories as well)
const ALGO: Algorithm = checksums::Algorithm::BLAKE2;
const JOBS: usize = 2;


fn main () {
    let mut cp_path: PathBuf = env::current_dir().unwrap();
    cp_path.push(TARGET_DIR);
    if !cp_path.exists() {
        println!("The '{}' directory doesn't exist; unable to create / update manifest.", TARGET_DIR);
        return;
    }
    let now: DateTime<Local> = Local::now();
    println!("Building manifest for {}...", cp_path.display());
    let ver_str = format!("{}.{}.{}.{}", now.year(), now.month(), now.day(), 1);
    let ver: Version = Version::from_string(&ver_str);
    println!("Version string would be: {:?}", ver);
    let ignores = BTreeSet::new();
    let max_recursion: Option<usize> = Some(10);
    let hashes: BTreeMap<String, String> = create_hashes(&cp_path,
        ignores,
        ALGO,
        max_recursion,
        true,
        JOBS,
        stdout(),
        &mut stderr()
    );
}
