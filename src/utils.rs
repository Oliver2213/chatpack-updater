// utilities


use std::path::Path;
use std::collections::BTreeSet;
use std::error::Error;
use checksums::util::relative_name;
use gitignore::File;
use walkdir::WalkDir;


/// Given a path and a `gitignore::File` instance, return a BTreeSet of ignored files (as strings relative to the path given), ready for passing to checksums::ops::create_hashes
pub fn ignored_files (path: &Path, ignore_file: &File) -> BTreeSet<String> {
    // the references in the returned BTreeSet will live as long as the git ignore file instance does
    
    // This function returns every file (as a string relative to the `path` arg)
    // this is for use with checksums, since it only allows you to provide a BTreeSet of strings
    // this has to walk `path` and check every file
    let mut ignores: BTreeSet<String> = BTreeSet::new();
    let mut walker = WalkDir::new(path).into_iter();
    while let Some(e) = walker.next() {
        match e {
            Ok(entry) => {
                // I don't do anything special if the entry is a directory, as checksums will skip it if provided
                // check to see if the path is excluded
                let ignored_entry = ignore_file.is_excluded(entry.path()).unwrap();
                if ignored_entry {
                    ignores.insert(relative_name(path, entry.path()));
                }
            },
            Err(e) => panic!("Error traversing directory: {}", e.description()),
        }
    }
    ignores
}
