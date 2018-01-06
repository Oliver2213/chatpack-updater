// constants needed by more than one component of the updater

use checksums::Algorithm;

pub const TARGET_DIR: &str = "chatpack";
pub const ALGO: Algorithm = Algorithm::BLAKE2;
pub const JOBS: usize = 2;
pub const VERSION_FILENAME :&str = "chatpack.ver"; // the name of the file (under target_dir) which holds chatpack's current version (and which needs to be updated by this program)
pub const MANIFEST_FILENAME: &str = "chatpack.update-manifest"; // The filename which contains the hash manifest (which this program will download and compare against)
pub const MASTER_MANIFEST_URL: &str = "https://raw.githubusercontent.com/ChatMUD/chatpack/master/chatpack.update-manifest";
