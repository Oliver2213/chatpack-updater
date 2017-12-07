// Chatpack updater in rust, by Blake Oliver <oliver22213@me.com>

use std::io::{stdout, stderr};
use std::path::{Path, PathBuf};
use std::env;
use std::collections::{BTreeSet, BTreeMap};
// pull in checksums
extern crate checksums;
use checksums::ops::create_hashes;
use checksums::Algorithm;


// set constants
const TARGET_DIR: &str = "chatpack";
const ALGO: Algorithm = checksums::Algorithm::BLAKE2;
const JOBS: usize = 2;


fn main () {
    let mut cp_path: PathBuf = env::current_dir().unwrap();
    cp_path.push(TARGET_DIR);
    println!("Using directory '{}'.", cp_path.display());
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
