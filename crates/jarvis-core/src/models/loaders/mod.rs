#[cfg(any(feature = "jarvis_app", feature = "intent"))]
pub mod embedding;
#[cfg(feature = "jarvis_app")]
pub mod gliner;
#[cfg(feature = "jarvis_app")]
pub mod ort_model;
#[cfg(any(feature = "jarvis_app", feature = "intent"))]
pub mod intent_classifier;
#[cfg(feature = "vosk")]
pub mod vosk;
#[cfg(feature = "nnnoiseless")]
pub mod nnnoiseless;
