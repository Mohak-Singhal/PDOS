use std::fs;
use std::path::PathBuf;

use dirs::home_dir;

use super::Identity;

fn identity_file() -> PathBuf {
    let mut path = if let Some(dir) = crate::APP_DATA_DIR.get() {
        PathBuf::from(dir)
    } else {
        home_dir().expect("Unable to locate home directory")
    };

    path.push(".pdos");

    fs::create_dir_all(&path).expect("Unable to create PDOS directory");

    path.push("identity.json");

    path
}

pub fn load() -> Identity {
    let path = identity_file();

    if path.exists() {
        let data = fs::read_to_string(&path).expect("Unable to read identity");

        return serde_json::from_str(&data).expect("Invalid identity file");
    }

    let identity = crate::identity::crypto::generate();

    save(&identity);

    identity
}

pub fn save(identity: &Identity) {
    let path = identity_file();
    log::info!("Identity file: {}", path.display());

    let json = serde_json::to_string_pretty(identity).unwrap();

    fs::write(path, json).expect("Unable to save identity");
}
