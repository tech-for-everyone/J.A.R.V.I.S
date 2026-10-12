pub mod noise_suppression;
pub mod vad;
pub mod gain_normalizer;

use once_cell::sync::OnceCell;
use parking_lot::Mutex;

use crate::config::structs::NoiseSuppressionBackend;
use crate::DB;

static PROCESSOR: OnceCell<Mutex<AudioProcessor>> = OnceCell::new();

/// Root-mean-square amplitude of 16-bit PCM samples.
///
/// Shared by the energy VAD and the gain normalizer to avoid duplicating the
/// same accumulation math in both.
pub fn calculate_rms(samples: &[i16]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }

    let sum: f64 = samples.iter().map(|&s| (s as f64) * (s as f64)).sum();
    (sum / samples.len() as f64).sqrt() as f32
}

#[derive(Debug, Clone)]
pub struct ProcessedAudio {
    pub samples: Vec<i16>,
    pub is_voice: bool,
    pub vad_confidence: f32,
}

struct AudioProcessor {
    has_gain: bool,
    has_ns: bool,
}

impl AudioProcessor {
    fn new(ns: NoiseSuppressionBackend, gain: bool) -> Self {
        noise_suppression::init(ns);
        vad::init();
        if gain {
            gain_normalizer::init();
        }

        Self {
            has_gain: gain,
            has_ns: !matches!(ns, NoiseSuppressionBackend::None),
        }
    }

    fn process(&mut self, input: &[i16]) -> ProcessedAudio {
        let gained: Vec<i16>;
        let after_gain: &[i16] = if self.has_gain {
            gained = gain_normalizer::normalize(input);
            &gained
        } else {
            input
        };

        let suppressed: Vec<i16>;
        let after_ns: &[i16] = if self.has_ns {
            suppressed = noise_suppression::process(after_gain);
            &suppressed
        } else {
            after_gain
        };

        let (is_voice, confidence) = vad::detect(after_ns);

        ProcessedAudio {
            samples: after_ns.to_vec(),
            is_voice,
            vad_confidence: confidence,
        }
    }

    fn reset(&mut self) {
        noise_suppression::reset();
        vad::reset();
        gain_normalizer::reset();
    }
}

pub fn init() -> Result<(), String> {
    if PROCESSOR.get().is_some() {
        return Ok(());
    }

    let (ns, gain) = get_settings();
    info!("Initializing audio processing: NS={:?}, Gain={}", ns, gain);

    let processor = AudioProcessor::new(ns, gain);
    PROCESSOR
        .set(Mutex::new(processor))
        .map_err(|_| "Audio processor already initialized".to_string())?;

    info!("Audio processing initialized.");
    Ok(())
}

pub fn process(input: &[i16]) -> ProcessedAudio {
    match PROCESSOR.get() {
        Some(p) => p.lock().process(input),
        None => ProcessedAudio {
            samples: input.to_vec(),
            is_voice: true,
            vad_confidence: 1.0,
        },
    }
}

pub fn reset() {
    if let Some(p) = PROCESSOR.get() {
        p.lock().reset();
    }
}

#[cfg(test)]
mod tests {
    use super::calculate_rms;

    #[test]
    fn rms_of_empty_is_zero() {
        assert_eq!(calculate_rms(&[]), 0.0);
    }

    #[test]
    fn rms_of_constant_signal_equals_amplitude() {
        let samples = vec![1000i16; 256];
        assert!((calculate_rms(&samples) - 1000.0).abs() < 0.01);
    }

    #[test]
    fn rms_of_sine_is_amplitude_over_sqrt2() {
        let amplitude = 10_000.0f32;
        let samples: Vec<i16> = (0..2048)
            .map(|i| {
                let t = i as f32 / 2048.0 * std::f32::consts::TAU;
                (t.sin() * amplitude) as i16
            })
            .collect();
        let expected = amplitude / std::f32::consts::SQRT_2;
        let rms = calculate_rms(&samples);
        assert!(
            (rms - expected).abs() < 50.0,
            "rms {} is not close to {}",
            rms,
            expected
        );
    }
}

fn get_settings() -> (NoiseSuppressionBackend, bool) {
    match DB.get() {
        Some(db) => {
            let settings = db.read();
            (settings.noise_suppression, settings.gain_normalizer)
        }
        None => (
            crate::config::DEFAULT_NOISE_SUPPRESSION,
            crate::config::DEFAULT_GAIN_NORMALIZER,
        ),
    }
}
