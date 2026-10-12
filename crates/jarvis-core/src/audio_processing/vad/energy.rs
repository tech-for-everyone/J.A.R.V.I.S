use crate::config;

// Simple energy-based VAD
pub fn detect(input: &[i16]) -> (bool, f32) {
    let rms = crate::audio_processing::calculate_rms(input);
    let is_voice = rms > config::VAD_ENERGY_THRESHOLD;

    // normalize confidence to 0-1 range (rough approximation)
    let confidence = (rms / (config::VAD_ENERGY_THRESHOLD * 2.0)).min(1.0);

    (is_voice, confidence)
}