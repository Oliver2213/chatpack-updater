// Chatpack updater in rust, by Blake Oliver <oliver22213@me.com>

// This program Hashes files under `TARGET_DIR`, then compares that to a downloaded manifest it retrieves from the repository, then replaces files who's hashes differ

use std::io::{stdout, stderr};
use std::fs::{OpenOptions, read_dir, create_dir_all, rename};
use std::path::PathBuf;
use std::env;
use std::collections::{BTreeSet, BTreeMap};
use std::error::Error;
// pull in checksums
extern crate checksums;
use checksums::ops::{create_hashes, compare_hashes, CompareResult, CompareFileResult};
use indicatif::{ProgressBar, ProgressStyle};

extern crate chatpack_updater; // pull in our library crate so every binary can use things it reexports
use chatpack_updater::utils;

extern crate reqwest;

extern crate gitignore;

// get constants
use chatpack_updater::constants::*;

fn main () {
    let cp_path: PathBuf = env::current_dir().unwrap();
    let mush_directory_markers: Vec<&str> = ["MUSHclient.exe", "worlds", "mushclient_prefs.sqlite"].to_vec(); // Files and directories that indicate a mush client directory
    // make sure this program is located inside a mush directory
    // do this by getting a list of `Path` instances for the current directory and looping through the marker paths to see if it contains one of them
    // do this in a different scope so we don't keep around variables that aren't needed afterwards
    {
        let entries: Vec<PathBuf> = read_dir(&cp_path).expect("Can't read files in current directory.")
          .map(| e | {
            match e {
                Ok(e) => e.path(),
                Err(why) => panic!("Can't read entry in current directory: {}", why.description()),
            } // end the match
          }) // end the closure
          .collect();
        let mut marker_found= false;
        let dir: PathBuf = env::current_dir().unwrap();
        for e in &mush_directory_markers {
            let mut e_path: PathBuf = dir.clone();
            e_path.push(&e);
            if entries.contains(&e_path) {
                marker_found = true
            }
        }
        if !marker_found {
            println!("You must run the {} updater from your mush folder.", TARGET_DIR);
            return;
        }
    }
    // identify the path to this program
    let this_prog_path = env::current_exe().expect("Unable to get the path to the updater.");
    let this_prog_name = this_prog_path.file_name().unwrap();
    // get a reqwest client instance (for http requests)
    let r_client = reqwest::Client::new();
    println!("Retrieving a snapshot of what files in the latest version look like...");
    // now, before doing any work hashing files, try to download the hash manifest from the repository that we'll need to compare against
    // do this in a different scope so we don't have variables sticking around that won't be used later
    // keep the `master_manifest` variable, though
    let master_manifest;
    {
        let mut resp = match r_client.get(MASTER_MANIFEST_URL).send() {
            Ok(r) => r,
            Err(why) => panic!("Can't retrieve the manifest file needed to update: {}", why.description()),
        };
        if !resp.status().is_success() {
            println!("Can't retrieve the manifest file needed to update: returned status code {}. Please try again later.", resp.status());
            return;
        }
        // now that we have a response, get the body and parse
        let j: BTreeMap<String, String> = resp.json().expect("Error parsing downloaded manifest file.");
        master_manifest = j;
    }
    println!("Done.");
    
    // find all ignored files and directories, so they can be skipped when hashing to save time
    let mut ignores: BTreeSet<String> = BTreeSet::new();
    let mut standard_ignores_path = cp_path.clone();
    standard_ignores_path.push(STANDARD_UPDATER_IGNORE_FILENAME);
    let mut custom_ignores_path = cp_path.clone();
    custom_ignores_path.push(CUSTOM_UPDATER_IGNORE_FILENAME);
    let mut ignore_files: Vec<gitignore::File> = vec!(); // a vec with gitignore file instances; each will be processed and combined
    if standard_ignores_path.exists() {
        ignore_files.push(gitignore::File::new(&standard_ignores_path).unwrap());
    }
    if custom_ignores_path.exists() {
        ignore_files.push(gitignore::File::new(&custom_ignores_path).unwrap())
    }
    if !ignore_files.is_empty(){
        // walk our current directory recursively and add relative paths of ignored files and dirs
        ignores.append(&mut utils::ignored_files(&cp_path, ignore_files));
    }
    let max_recursion: Option<usize> = Some(10);
    println!("Taking a snapshot of how files look now...");
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
    println!();
    println!("Done.");
    println!("Determining what files need updating...");
    // now compare them against the downloaded manifest
    let res = compare_hashes("", master_manifest, hashes);
    let mut new_files: Vec<String>;
    let mut modified_files: Vec<String>;
    let mut ignored_files: Vec<String>;
    let mut removed_files: Vec<String>;
    match res {
        Ok((cr, fcr)) => {
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
    println!("Done. {} new files, {} modified files.", new_files.len(), modified_files.len());
    
    // Now download the files that are new or have been modified
    let mut ftd = vec![]; // files to download
    ftd.extend(new_files);
    ftd.extend(modified_files);
    let total = ftd.len();
    // now that all the pathstrings are in one vec, loop over them, percent encode them, download them, then write each to disk
    // Oh, and progress bar too.
    let download_progbar = indicatif::ProgressBar::new(total as u64);
    download_progbar.set_style(
      ProgressStyle::default_bar()
      .template("{pos}/{len} - {msg} Remaining: {eta} {bar:>}")
      .progress_chars("#>-")
    );
    for pathstring in ftd {
        download_progbar.set_message(&pathstring);
        download_progbar.inc(1);
        let e = utils::percent_encode_pathstring(&pathstring);
        //println!("Downloading {:?}", e);
        let mut url: &str = &format!("{}{}", BASE_FILE_URL, e);
        //println!("URL: {:?}", url);
        let mut resp = match r_client.get(url).send() {
            Ok(response) => response,
            Err(why) => panic!("Can't retrieve file '{}': {}", pathstring, why.description()),
        };
        // We have a valid response; check it's status code
        if !resp.status().is_success(){
            println!("Error retrieving file '{}': returned status code {}. Please try updating again later.", pathstring, resp.status());
            return;
        }
        // the program has exited if the status wasn't a success; now write the file out to disk.
        {
            let mut p: PathBuf = cp_path.clone();
            p.push(&pathstring);
            // since open won't create intervening directories, run create_dir_all on path.parent to create any directories up the file that don't exist
            create_dir_all(&p.parent().unwrap()).unwrap();
            // check if the file to be updated is actually this program
            if this_prog_name == p.file_name().unwrap() {
                // rename this program, passing a '.old' suffix so the new version can be downloaded.
                rename(&this_prog_path, &this_prog_path.with_extension("old")).expect("Error renaming the updater.");
            }
            let f = OpenOptions::new().create(true).write(true).truncate(true).open(&p);
            match f {
                Ok(mut file) => {
                    //println!("Writing to disk...");
                    // copy the http response (which is the file) to the opened file
                    resp.copy_to(&mut file).unwrap();
                    //println!("Done!");
                },
                Err(why) => {
                    println!("Unable to open file '{}': {}", p.display(), why.description());
                    return;
                },
            }
        } // end the file write scope
    }// end the pathstring for loop
    download_progbar.finish_with_message(&format!("downloaded"));
    println!("Update completed!");
}
