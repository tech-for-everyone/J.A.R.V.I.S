use std::path::Path;

use once_cell::sync::OnceCell;
use porcupine::{Porcupine, PorcupineBuilder};

use crate::config;
use crate::DB;

// store porcupine instance
static PORCUPINE: OnceCell<Porcupine> = OnceCell::new();

pub fn init() -> Result<(), ()> {
    // retrieve picovoice api key
    let picovoice_api_key = match DB.get() {
        Some(settings) => settings.read().api_keys.picovoice.clone(),
        None => {
            error!("Settings are not initialized; cannot read the Picovoice API key.");
            return Err(());
        }
    };
    if picovoice_api_key.trim().is_empty() {
        warn!("Picovoice API key is not set.");
        return Err(());
    }

    // create porcupine instance with the given API key
    match PorcupineBuilder::new_with_keyword_paths(
        picovoice_api_key,
        &[Path::new(config::KEYWORDS_PATH).join(config::DEFAULT_KEYWORD)],
    )
    .sensitivities(&[config::DEFAULT_SENSITIVITY]) // set sensitivity
    .init()
    {
        Ok(pinstance) => {
            // success
            info!("Porcupine successfully initialized with the given API key.");

            // store
            PORCUPINE.set(pinstance);
        }
        Err(msg) => {
            error!("Porcupine failed to initialize, either API key is not valid or there is no internet connection.");
            error!("Error details: {}", msg);

            return Err(());
        }
    }

    Ok(())
}

pub fn data_callback(frame_buffer: &[i16]) -> Option<i32> {
    let porcupine = PORCUPINE.get()?;

    if let Ok(keyword_index) = porcupine.process(frame_buffer) {
        if keyword_index >= 0 {
            return Some(keyword_index);
        }
    }

    None
}
