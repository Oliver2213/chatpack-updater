// library crate for chatpack-updater

// This doesn't really provide anything other people would be interested in, so probably won't be published to cargo;
// I'm just using this as a convenient place to pull in modules, add utilities, and let src/main.rs as well as extra binaries in src/bin/*.rs use them

pub mod version;
pub mod constants;
pub mod utils;

extern crate chrono;
extern crate checksums;
extern crate gitignore;
extern crate walkdir;
extern crate url;
