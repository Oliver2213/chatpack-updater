// constants needed by more than one component of the updater

use checksums::Algorithm;

pub const TARGET_DIR: &str = "chatpack";
pub const ALGO: Algorithm = Algorithm::BLAKE2;
pub const JOBS: usize = 2;
pub const VERSION_FILENAME :&str = "chatpack.ver"; // the name of the file (under target_dir) which holds chatpack's current version (and which needs to be updated by this program)
pub const MANIFEST_FILENAME: &str = "chatpack.update-manifest"; // The filename which contains the hash manifest (which this program will download and compare against)
pub const MASTER_MANIFEST_URL: &str = "https://git.chatmud.com/athlon/chatpack/raw/master/chatpack.update-manifest";
pub const STANDARD_UPDATER_IGNORE_FILENAME: &str = "chatpack-standard.update-ignore"; // the name of the file with git ignore syntax specifying files and directories all components of the updater will ignore by default
pub const CUSTOM_UPDATER_IGNORE_FILENAME: &str = "chatpack-custom.update-ignore"; // the name of the file with git ignore syntax specifying files and directories all components of the updater will ignore; this is meant for use by the user, and gets applied after the standard patterns
pub const BASE_FILE_URL: &str = "https://git.chatmud.com/athlon/chatpack/raw/master/chatpack/"; // base url which individual filenames will get added to
