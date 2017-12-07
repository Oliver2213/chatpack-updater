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


// set constants
const TARGET_DIR: &str = "chatpack";
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
    let ver = format!("{}.{}.{}.{}", now.year(), now.month(), now.day(), 1);
    println!("Version string would be: {}", ver);
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
    //println!("Hello, world!");
}
