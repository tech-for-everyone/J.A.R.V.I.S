pub mod structs;
pub mod manager;

use crate::{config, APP_CONFIG_DIR, APP_DIR};

use log::{info, warn};
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

pub use manager::SettingsManager;

fn get_db_file_path() -> PathBuf {
    match APP_CONFIG_DIR.get() {
        Some(config_dir) => config_dir.join(config::DB_FILE_NAME),
        None => {
            // init_dirs() was not called; fall back to the application directory
            warn!("Config directory not set, storing settings next to the executable");
            APP_DIR.join(config::DB_FILE_NAME)
        }
    }
}

pub fn init_settings() -> structs::Settings {
    let db_file_path = get_db_file_path();

    info!(
        "Loading settings db file located at: {}",
        db_file_path.display()
    );

    if db_file_path.exists() {
        if let Ok(db_file) = File::open(&db_file_path) {
            let reader = BufReader::new(db_file);
            if let Ok(settings) = serde_json::from_reader(reader) {
                info!("Settings loaded.");
                return settings;
            }
        }
    }

    warn!("No settings file found or there was an error parsing it. Creating default struct.");
    structs::Settings::default()
}

/// init settings and return a SettingsManager ready to use
pub fn init() -> SettingsManager {
    let settings = init_settings();
    SettingsManager::new(settings)
}

pub fn save_settings(settings: &structs::Settings) -> Result<(), std::io::Error> {
    let db_file_path = get_db_file_path();

    std::fs::write(
        &db_file_path,
        serde_json::to_string_pretty(&settings).unwrap(),
    )?;

    info!("Settings saved to: {:#}", db_file_path.display());
    Ok(())
}
