// Chatpack updater in rust, by Blake Oliver <oliver22213@me.com>

// This program Hashes files under `TARGET_DIR`, then compares that to a downloaded manifest it retrieves from the repository, then replaces files who's hashes differ

use std::io::{stdout, stderr, Read};
use std::fs::{File, OpenOptions};
use std::path::PathBuf;
use std::env;
use std::collections::{BTreeSet, BTreeMap};
use std::error::Error;
// pull in checksums
extern crate checksums;
use checksums::ops::{create_hashes, compare_hashes, CompareResult, CompareFileResult};
use checksums::Algorithm;

extern crate chatpack_updater; // pull in our "library" crate so every binary can use things it reexports
extern crate reqwest;


// set constants
const TARGET_DIR: &str = "chatpack";
const ALGO: Algorithm = checksums::Algorithm::BLAKE2;
const JOBS: usize = 2;
const VERSION_FILENAME :&str = "chatpack.ver"; // the name of the file (under target_dir) which holds chatpack's current version (and which needs to be updated by this program)
const MANIFEST_FILENAME: &str = "chatpack.update-manifest"; // The filename which contains the hash manifest (which this program will download and compare against)
const MASTER_MANIFEST_URL: &str = "https://raw.githubusercontent.com/ChatMUD/chatpack/master/chatpack.update-manifest";


fn main () {
    let cp_path: PathBuf = env::current_dir().unwrap();
    // make sure this program is located inside `TARGET_DIR`
    // I could turn off the must_use thing and then I could just do get_filename without a match (as that is unlikely to ever fail)
    // but I can't remember what it's called exactly, so...
    match cp_path.file_name() {
        Some(dirname) => {
            match dirname.to_str() {
                Some(s) => {
                    if s != TARGET_DIR{
                        println!("This updater must be run from inside the '{}' directory.", TARGET_DIR);
                        return;
                    }
                },
                // this is a pretty obscure error but...
                None => panic!("Can't decode current directory name from an OS String."),
            }
        },
        None => panic!("Can't determine current directory name."),
    }
    // get a reqwest client instance (for http requests)
    let r_client = reqwest::Client::new();
    // now, before doing any work hashing files, try to download the hash manifest from the repository that we'll need to compare against
    // do this in a different scope so we don't have variables sticking around that won't be used later
    // keep the `master_manifest` variable, though
    let master_manifest;
    {
        let mut resp = match r_client.get(MASTER_MANIFEST_URL).send() {
            Ok(r) => r,
            Err(why) => panic!("Can't retrieve the manifest file needed to update: {}", why.description()),
        };
        if resp.status().is_success() == false {
            panic!("Can't retrieve the manifest file needed to update: git hub returned status code {}.", resp.status())
        }
        // now that we have a response, get the body and parse
        let j: BTreeMap<String, String> = resp.json().expect("Error parsing downloaded manifest file.");
        master_manifest = j;
    }
    // this will be populated later, when I add the ability for users to ignore files (so the updater won't update them), but for now it's just an empty set
    let ignores = BTreeSet::new();
    let max_recursion: Option<usize> = Some(10);
    // Hash files in `TARGET_DIR` to determine what needs to be updated
    let hashes: BTreeMap<String, String> = create_hashes(&cp_path,
        ignores,
        ALGO,
        max_recursion,
        true,
        JOBS,
        stdout(),
        &mut stderr()
    );
    // now compare them against the downloaded manifest
    let res = compare_hashes("", hashes, master_manifest);
    let mut new_files: Vec<String>;
    let mut modified_files: Vec<String>;
    let mut ignored_files: Vec<String>;
    let mut removed_files: Vec<String>;
    match res {
        Ok((mut cr, mut fcr)) => {
            // initialize the variables that need to survive out of this scope
            // without the next 4 lines, the compiler yells about using uninitialized variables so
            new_files = [].to_vec();
            modified_files = [].to_vec();
            ignored_files = [].to_vec();
            removed_files = [].to_vec();
            for r in &cr {
                match *r {
                    CompareResult::FileAdded(ref file) => new_files.push(file.to_owned()),
                    CompareResult::FileRemoved(ref file) => removed_files.push(file.to_owned()),
                    CompareResult::FileIgnored(ref file) => ignored_files.push(file.to_owned()),
                } // end the match
            } // end the for loop
            // now check for modified files
            for r in &fcr {
                match *r {
                    CompareFileResult::FileMatches(_) => (), // don't do anything if files are the same
                    CompareFileResult::FileDiffers {ref file, ref was_hash, ref new_hash} => modified_files.push(file.to_owned()),
                } // end individual file match
            } // end the for loop
        } // end ok
        Err(_) => panic!("Error comparing hashes: hash lengths of the downloaded manifest and locally-generated one differ."),
    }
    println!("Comparison results: {} new files, {} modified files, {} removed files.", new_files.len(), modified_files.len(), removed_files.len());
}
