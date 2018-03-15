// Chatpack manifest updater in rust, by Blake Oliver <oliver22213@me.com>

// This program hashes files under TARGET_DIR and outputs the result to a file;
// That file is for use by the actual updater

use std::io::{stdout, stderr, SeekFrom};
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

extern crate chatpack_updater;
use chatpack_updater::version::Version;
use chatpack_updater::utils;

extern crate serde_json;

extern crate gitignore;

// get constants
use chatpack_updater::constants::*;


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
    // update the version        
    // do this first so the generated file gets added to the hash manifest
    // since the lines below are in different scopes (and thus variables defined won't be valid outside of them), I declare the ones I want to keep here
    // clone cp_path (so it's a new object, rather than taking ownership) so cp_version_path can add to it
    let mut cp_version_path = cp_path.clone();
    cp_version_path.push(VERSION_FILENAME);
    // Check to see if there's an existing version file
    if cp_version_path.exists() == false {
        println!("{}'s version file not found; one will be created.", TARGET_DIR);
    }
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
                str_ver = str_ver.trim().into();
                if str_ver.is_empty() {
                    // file is empty for some reason; use the current date-based version
                    println!("Warning: the version file is empty; using a new version string because an old one doesn't exist to update.");
                    version = Version::new();
                } else {
                    version = Version::from_string(&str_ver);
                    version.update();
                }
                // delete the current contents of the file so it can be written to later with just the newly-created version as a string
                file.seek(SeekFrom::Start(0)).expect("Can't seek to offset 0 in the version file to overwrite it.");
                match file.set_len(0){
                    Ok(_) => (),
                    Err(why) => panic!("Can't overwrite {}: {}", cp_version_path.display(), why.description()),
                }
                // return version_file
                file
            },
            Err(why) => panic!("Can't read from {}: {}", cp_version_path.display(), why.description()),
        };
    } // end the exists if block
    match version_file.write_all(version.to_string().as_bytes()) {
        Ok(_) => version_file.flush().expect("Can't write version out to file."),
        Err(why) => panic!("Can't write new version to {}: {}", cp_version_path.display(), why.description()),
    }
    
    // check if there's an existing hash manifest
    let mut cp_manifest_path: PathBuf = env::current_dir().unwrap();
    cp_manifest_path.push(&MANIFEST_FILENAME);
    if cp_manifest_path.exists() == false {
        println!("Creating initial {} manifest...", TARGET_DIR);
    } else {
        println!("Rebuilding {}'s manifest...", TARGET_DIR);
    }
    let mut ignores: BTreeSet<String> = BTreeSet::new();
    let mut standard_ignores_path = cp_path.clone();
    standard_ignores_path.push(STANDARD_UPDATER_IGNORE_FILENAME);
    let mut custom_ignores_path = cp_path.clone();
    custom_ignores_path.push(CUSTOM_UPDATER_IGNORE_FILENAME);
    let mut ignore_files: Vec<gitignore::File> = vec!();
    if standard_ignores_path.exists() {
        ignore_files.push(gitignore::File::new(&standard_ignores_path).unwrap());
    }
    if custom_ignores_path.exists() {
        ignore_files.push(gitignore::File::new(&custom_ignores_path).unwrap())
    }
    if ignore_files.is_empty() == false {
        // walk our current directory recursively and add relative paths of ignored files and dirs
        ignores.append(&mut utils::ignored_files(&cp_path, ignore_files));
    }
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
    
    // We've now created then written a manifest out to disk, then updated the version
    // Now check to see if this program is being used as a git hook, and if so, add the manifest and version files to git's index (so they automatically get included in commits)
    match env::current_exe() {
        Ok(p) => {
            // successfully got path
            let name = match p.file_name() {
                Some(n) => n.to_str().unwrap(), // if we got the last component, return it (as an str)
                None => panic!("Can't parse executable name of this program for the git hook check. This means I can't automatically add the generated hash manifest and version file to the git index if this program is being used as a pre-commit git hook; if this is the case, you'll need to manually git add them."),
            };
            let args = env::args().collect::<Vec<String>>();
            if name == "pre-commit" || args.contains(&"pre-commit".to_string()) {
                // this program is being used as a git hook, which means we should add our previously-generated hash manifest and version files to it's index, so they get automatically committed
                let git_add_version_status = Command::new("git")
                    .args(&["add", cp_version_path.to_str().unwrap()])
                    .status()
                    .expect("Can't run `git add` to add the version file to git's index");
                let git_add_manifest_status = Command::new("git")
                    .args(&["add", cp_manifest_path.to_str().unwrap()])
                    .status()
                    .expect("Can't run `git add` to add the hash manifest to git's index");
                if git_add_version_status.success() == false || git_add_manifest_status.success() == false {
                    // the code returned by either of the git add commands was non-zero
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
