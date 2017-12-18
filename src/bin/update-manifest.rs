// Chatpack manifest updater in rust, by Blake Oliver <oliver22213@me.com>

// This program hashes files under TARGET_DIR and outputs the result to a file;
// That file is for use by the actual updater

use std::io::{stdout, stderr};
use std::path::PathBuf;
use std::fs::{File, OpenOptions};
use std::error::Error;
use std::io::prelude::*;
use std::env;
use std::collections::{BTreeSet, BTreeMap};
use std::process::Command;
// pull in checksums
extern crate checksums;
use checksums::ops::create_hashes;
use checksums::Algorithm;

extern crate chatpack_updater;
use chatpack_updater::version::Version;

extern crate serde_json;

// set constants
const TARGET_DIR: &str = "chatpack"; // The directory (under the current working one) in which files will be hashed (recursing into subdirectories as well); this is also used as the name in messages that get printed
const ALGO: Algorithm = checksums::Algorithm::BLAKE2; // what hashing algorithm should be used
const JOBS: usize = 2;
const VERSION_FILENAME :&str = "chatpack.ver"; // the name of the file (under target_dir) which holds chatpack's current version (and which needs to be updated by this program)
const MANIFEST_FILENAME: &str = "chatpack.update-manifest"; // The filename which the hash manifest gets written to (in the current working directory for this program, not under TARGET_DIR)


fn main () {
    // set the chatpack path variable to the current working directory
    let mut cp_path: PathBuf = env::current_dir().unwrap();
    // then add `target_dir` to that, making `cp_path` the full path to the chatpack directory
    cp_path.push(&TARGET_DIR);
    // check if the chatpack directory exists, and return if it doesn't (as there's no work for us to do)
    if !cp_path.exists() {
        println!("The '{}' directory doesn't exist; unable to create / update manifest.", TARGET_DIR);
        return;
    }
    let mut cp_manifest_path: PathBuf = env::current_dir().unwrap();
    cp_manifest_path.push(&MANIFEST_FILENAME);
    // check if there's an existing hash manifest
    if cp_manifest_path.exists() == false {
        println!("Creating initial {} manifest...", TARGET_DIR);
    } else {
        println!("Rebuilding {}'s manifest...", TARGET_DIR);
    }
    // clone cp_path (so it's a new object, rather than taking ownership) so cp_version_path can add to it
    let mut cp_version_path = cp_path.clone();
    cp_version_path.push(VERSION_FILENAME);
    // Check to see if there's an existing version file
    if cp_version_path.exists() == false {
        println!("{}'s version file not found; one will be created.", TARGET_DIR);
    }
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
    //open the manifest file
    let manifest_file = File::create(&cp_manifest_path); // this is actually a `Result` struct
    match manifest_file {
        Ok(mut manifest_file) => {
            // now that the file is open, write to it
            match manifest_file.write_all(j.as_bytes()) {
                Err(why) => {
                    panic!("Couldn't write to {}: {}", cp_version_path.display(), why.description());
                }
                Ok(_) => println!("Manifest written out to '{}'.", cp_manifest_path.display()),
            }
        }, // end of the ok block
        Err(why) => panic!("couldn't create or open {}: {}", cp_manifest_path.display(), why.description()),
    } // end of the manifest file open error checking block
    //now update the version        
    // since the lines below are in different scopes (and thus variables defined won't be valid outside of them), I declare the ones I want to keep here
    let mut version_file;
    let mut version;
    // if a version file doesn't exist, create one and populate it with the current version (based on the date)
    // Otherwise, read what does exist, convert it to a `Version`, and .update() it
    if cp_version_path.exists() == false {
        version_file = match File::create(&cp_version_path) {
            Err(why) => panic!("couldn't create {}: {}", cp_version_path.display(), why.description()),
            Ok(file) => file,
        };
        // get current date-based version, for use further down
        version = Version::new();
    } else {
        // the version file exists, so we need to read it into a string, convert that to a `Version`, run .update(), and then seek to the start of the file so it can be written out
        version_file = match OpenOptions::new().read(true).write(true).open(&cp_version_path) {
            Ok(mut file) => {
                let mut str_ver: String = String::new();
                // now read all the bytes in the opened file into a string
                match file.read_to_string(&mut str_ver) {
                    Ok(_) => (),
                    Err(why) => panic!("Can't read version file {}: {}", cp_version_path.display(), why.description()),
                }
                println!("Read bytes from version file. Contents: '{}'", str_ver);
                str_ver = str_ver.trim_left().parse().expect("Failed to interpret version string as an str after trim.");
                println!("After a trim, we have: '{}'", str_ver);
                version = Version::from_string(&str_ver);
                version.update();
                // delete the current contents of the file so it can be written to later with just the newly-created version as a string
                // put the following in a match statement,
                match file.set_len(0){
                    Ok(_) => (),
                    Err(why) => panic!("Can't overwrite {}: {}", cp_version_path.display(), why.description()),
                }
                // return version_file
                file
            },
            Err(why) => panic!("Can't write to {}: {}", cp_version_path.display(), why.description()),
        };
    } // end the exists if block
    match version_file.write_all(version.to_string().as_bytes()) {
        Ok(_) => (),
        Err(why) => println!("Can't write new version to {}: {}", cp_manifest_path.display(), why.description()),
    }

    // We've now created then written a manifest out to disk, then updated the version
    // Now check to see if this program is being used as a git hook, and if so, add the manifest and version files to git's index (so they automatically get included in commits)
    match env::current_exe() {
        Ok(p) => {
            // successfully got path
            let name = match p.file_name() {
                Some(n) => n.to_str().unwrap(), // if we got the last component, return it (as an str)
                None => panic!("Can't parse executable name of this program for the git hook check. This means I can't automatically add the generated hash manifest and version file to the git index if this program is being used as a pre-commit git hook; if this is the case, you'll need to manually git add them."),
            };
            if name == "pre-commit" {
                // this program is being used as a git hook, which means we should add our previously-generated hash manifest and version files to it's index, so they get automatically committed
                let git_add_status = Command::new("git")
                    .args(&["add", cp_manifest_path.to_str().unwrap(), cp_version_path.to_str().unwrap()])
                    .status()
                    .expect("Can't run `git add` to add the hash manifest and version files to git's index");
                if git_add_status.success() == false {
                    // the code returned by the git add command was non-zero
                    println!("Can't execute `git add`: this program is being used as a git pre-commit hook, but it's unable to automatically add the manifest and version files to git's index. You will have to do this manually before you commit with the following command: git add {} {}", cp_manifest_path.display(), cp_version_path.display());
                    return;
                } else {
                    // files added
                    println!("Hash manifest and version files have been added to git's index; ready to commit.");
                }
            }
        },
        Err(why) => panic!("Can't get path to this program:{}", why.description()),
    }
}
