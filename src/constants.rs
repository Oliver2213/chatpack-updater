// constants needed by more than one component of the updater

use checksums::Algorithm;

pub const TARGET_DIR: &str = "MUSHclient";
pub const ALGO: Algorithm = Algorithm::BLAKE2;
pub const JOBS: usize = 2;
pub const VERSION_FILENAME :&str = "erion-pack.ver"; // the name of the file (under target_dir) which holds chatpack's current version (and which needs to be updated by this program)
pub const MANIFEST_FILENAME: &str = "erion-pack.update-manifest"; // The filename which contains the hash manifest (which this program will download and compare against)
pub const MASTER_MANIFEST_URL: &str = "https://gitlab.com/erion1/soundpack/raw/master/erion-pack.update-manifest";
pub const STANDARD_UPDATER_IGNORE_FILENAME: &str = "erion-pack-standard.update-ignore"; // the name of the file with git ignore syntax specifying files and directories all components of the updater will ignore by default
pub const CUSTOM_UPDATER_IGNORE_FILENAME: &str = "erion-pack-custom.update-ignore"; // the name of the file with git ignore syntax specifying files and directories all components of the updater will ignore; this is meant for use by the user, and gets applied after the standard patterns
pub const BASE_FILE_URL: &str = "https://gitlab.com/erion1/soundpack/raw/master/MUSHclient/"; // base url which individual filenames will get added to
