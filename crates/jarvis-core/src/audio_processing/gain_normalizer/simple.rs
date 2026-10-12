use crate::config;

pub struct GainNormalizer {
    current_gain: f32,
}

impl GainNormalizer {
    pub fn new() -> Self {
        Self { current_gain: 1.0 }
    }

    pub fn normalize(&mut self, input: &[i16]) -> Vec<i16> {
        let rms = crate::audio_processing::calculate_rms(input);
        
        if rms < 1.0 {
            return input.to_vec();
        }

        let target_gain = config::GAIN_TARGET_RMS / rms;
        let clamped_gain = target_gain.clamp(config::GAIN_MIN, config::GAIN_MAX);

        self.current_gain = self.current_gain * 0.9 + clamped_gain * 0.1;

        input.iter()
            .map(|&s| {
                let amplified = (s as f32) * self.current_gain;
                amplified.clamp(i16::MIN as f32, i16::MAX as f32) as i16
            })
            .collect()
    }

    pub fn reset(&mut self) {
        self.current_gain = 1.0;
    }
}