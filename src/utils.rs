// utilities


use std::collections::BTreeSet;
use checksums::util::relative_name;
use gitignore::File;
use walkdir::WalkDir;


/// Given a path and a `gitignore::File` instance, return a BTreeSet of ignored files (as strings relative to the path given), ready for passing to checksums::ops::create_hashes
fn ignored_files(path: &Path, ignore_file: &File) -> BTreeSet<&str> {
    // This function returns every file (as a string relative to the `path` arg)
    // this is for use with checksums, since it only allows you to provide a BTreeSet of strings
    // this has to walk `path` and check every file
    let ignores: BTreeSet<&str> = BTreeSet::new();
    let walker = WalkDir::new(path).into_iter();
    while let Some(e) = walker.next() {
        match e {
            Ok(entry) => {
                // I don't do anything special if the entry is a directory, as checksums will skip it if provided
                // check to see if the path is excluded
                ignored_entry = ignore_file.is_excluded(entry).unwrap();
                if ignored_entry {
                    ignores.insert(relative_path(path, entry.path());
                }
            },
            Err(e) => panic!("Error traversing directory: {}", e.description()),
        }
    }
    ignores
}
