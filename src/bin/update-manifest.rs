// Chatpack manifest updater in rust, by Blake Oliver <oliver22213@me.com>

// This program hashes files under TARGET_DIR and outputs the result to a file;
// That file is for use by the actual updater

use std::io::{stdout, stderr};
use std::path::PathBuf;
use std::fs::File;
use std::error::Error;
use std::io::prelude::*;
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

extern crate serde_json;

// set constants
const TARGET_DIR: &str = "chatpack"; // The directory (under the current working one) in which files will be hashed (recursing into subdirectories as well); this is also used as the name in messages that get printed
const ALGO: Algorithm = checksums::Algorithm::BLAKE2; // what hashing algorithm should be used
const JOBS: usize = 2;
const VERSION_FILENAME :&str = "chatpack.ver"; // the name of the file (under target_dir) which holds chatpack's current version (and which needs to be updated by this program)


fn main () {
    // set the chatpack path variable to the current working directory
    let mut cp_path: PathBuf = env::current_dir().unwrap();
    // then add `target_dir` to that, making `cp_path` the full path to the chatpack directory
    cp_path.push(TARGET_DIR);
    if !cp_path.exists() {
        println!("The '{}' directory doesn't exist; unable to create / update manifest.", TARGET_DIR);
        return;
    }
    // Check to see if there's an existing version file
    // clone cp_path (so it's a new object, rather than taking ownership) so cp_version_path can add to it
    let mut cp_version_path = cp_path.clone();
    cp_version_path.push(VERSION_FILENAME);
    if cp_version_path.exists() == false {
        println!("{}'s version file not found; one will be created.", TARGET_DIR);
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
    // now catch json errors with a match
    let j = match serde_json::to_string(&hashes) {
        Err(why) => panic!("Couldn't create a json representation of the hash manifest: {}", why.description()),
        Ok(jstr) => jstr,
    };
    let mut version_file = match File::create(&cp_version_path) {
        Err(why) => panic!("couldn't create {}: {}", cp_version_path.display(), why.description()),
        Ok(file) => file,
    };
    // now that the file is open, write to it
    match version_file.write_all(j.as_bytes()) {
        Err(why) => {
            panic!("Couldn't write to {}: {}", cp_version_path.display(), why.description());
        }
        Ok(_) => println!("Manifest written out to '{}'.", cp_version_path.display()),
    }
}
