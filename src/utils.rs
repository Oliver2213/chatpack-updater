// utilities


use std::path::Path;
use std::collections::BTreeSet;
use std::error::Error;
use checksums::util::relative_name;
use gitignore::File;
use walkdir::WalkDir;
use url::percent_encoding::{utf8_percent_encode, DEFAULT_ENCODE_SET};


/// Given a path and a list of `gitignore::File` instances, return a BTreeSet of ignored files (as strings relative to the path given), ready for passing to checksums::ops::create_hashes
pub fn ignored_files (path: &Path, ignore_files: Vec<File>) -> BTreeSet<String> {
    // This function returns every file (as a string relative to the `path` arg)
    // this is for use with checksums, since it only allows you to provide a BTreeSet of strings
    // this has to walk `path` and check every file
    /*
This is really inefficient and I hate it, but...
gitignore has an included_files method, which is_excluded calls every time. So I can't use is_excluded, because that would trigger a recursion on every call.
So instead we call included_files, and check if each path I'm iterating over is in that list, which is what is_excluded does anyway; I'm just not needlessly recursing.
    */
    let mut ignores: BTreeSet<String> = BTreeSet::new();
    let mut included_files = vec![];
    for f in &ignore_files {
        let r = f.included_files().unwrap();
        included_files.extend(r);
        included_files.sort();
    }
    // now remove duplicate paths from the vec
    included_files.dedup();
    let mut walker = WalkDir::new(path).max_open(15).into_iter();
    while let Some(e) = walker.next() {
        match e {
            Ok(entry) => {
//                println!("Working on '{:?}'", entry.file_name());
                // check to see if the path is excluded for each ignore file we were given
                if included_files.contains(&entry.path().to_path_buf()) == false && entry.path() != path{
                    println!("Ignoring: {:?}", entry.path());
                    ignores.insert(relative_name(path, entry.path()));
                    // if this entry is  a directory, we can skip recursing through it, as gitignore says there's no included files here
                    if entry.file_type().is_dir() {
                        println!("Skipping directory {:?}", entry.path());
                        walker.skip_current_dir();
                    }
                }
/*
                    for f in &ignore_files {
                        let ignored_entry = f.is_excluded(entry.path()).unwrap();
                        if ignored_entry && entry.path() != path {
                            ignores.append(relative_name(path, entry.path()));
                        }
                    }
*/
            },
            Err(e) => panic!("Error traversing directory: {}", e.description()),
        }
    }    
    ignores
}

/// Given a string, split it up on the / character, url percent encode each substring, then reassenble them
pub fn percent_encode_pathstring (pathstring: &str) -> String {
    // This takes paths as strings rather than path objects
    let substrings: Vec<&str> = pathstring.split('/').collect();
    let mut encoded_subs: Vec<String>= Vec::new(); // vec to hold encoded substrings
    for sub in substrings {
        let i = utf8_percent_encode(sub, DEFAULT_ENCODE_SET);
        let encoded_sub: String = i.collect();
        encoded_subs.push(encoded_sub);
    }
    let result= encoded_subs.join("/");
    result
}
