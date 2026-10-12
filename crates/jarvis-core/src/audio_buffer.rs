use std::collections::VecDeque;

/// A bounded FIFO of audio frames.
///
/// Frames are stored as owned `Vec<i16>`; when the buffer is full the oldest
/// frame is dropped, so it always holds at most `max_frames` frames.
pub struct AudioRingBuffer {
    buffer: VecDeque<Vec<i16>>,
    max_frames: usize,
}

impl AudioRingBuffer {
    /// Create a buffer holding `seconds` worth of audio at the given
    /// `frame_size` (samples per frame) and `sample_rate` (samples per second).
    ///
    /// The capacity is always at least one frame, even for very small
    /// `seconds` values.
    pub fn new(seconds: f32, frame_size: usize, sample_rate: usize) -> Self {
        let max_frames = if frame_size == 0 || seconds <= 0.0 {
            1
        } else {
            let frames_per_second = sample_rate as f64 / frame_size as f64;
            ((frames_per_second * seconds as f64).ceil() as usize).max(1)
        };

        Self {
            buffer: VecDeque::with_capacity(max_frames),
            max_frames,
        }
    }

    /// Maximum number of frames this buffer can hold.
    pub fn capacity(&self) -> usize {
        self.max_frames
    }

    /// Push a frame, dropping the oldest one if the buffer is full.
    pub fn push(&mut self, frame: &[i16]) {
        if self.buffer.len() >= self.max_frames {
            self.buffer.pop_front();
        }
        self.buffer.push_back(frame.to_vec());
    }

    /// Remove and return all buffered frames, oldest first.
    pub fn drain_all(&mut self) -> Vec<Vec<i16>> {
        self.buffer.drain(..).collect()
    }

    /// Number of frames currently buffered.
    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    /// Returns `true` if no frames are buffered.
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    /// Drop all buffered frames.
    pub fn clear(&mut self) {
        self.buffer.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::AudioRingBuffer;

    #[test]
    fn capacity_is_at_least_one() {
        // tiny duration / zero frame size must not produce a zero frame and
        // panic on push
        let mut buf = AudioRingBuffer::new(0.0, 512, 16000);
        assert_eq!(buf.capacity(), 1);
        buf.push(&[1, 2, 3]);
        assert_eq!(buf.len(), 1);

        let buf = AudioRingBuffer::new(1.0, 0, 16000);
        assert_eq!(buf.capacity(), 1);
    }

    #[test]
    fn keeps_only_capacity_frames() {
        // 16000 / 512 = 31.25 -> 32 frames in one second
        let mut buf = AudioRingBuffer::new(1.0, 512, 16000);
        assert_eq!(buf.capacity(), 32);

        for i in 0..100i16 {
            buf.push(&[i]);
        }

        assert_eq!(buf.len(), 32);
        let frames = buf.drain_all();
        assert_eq!(frames.len(), 32);
        // oldest frames were dropped, so the first retained frame is 68
        assert_eq!(frames[0], vec![68]);
        assert_eq!(buf.len(), 0);
        assert!(buf.is_empty());
    }

    #[test]
    fn clear_empties_the_buffer() {
        let mut buf = AudioRingBuffer::new(1.0, 512, 16000);
        buf.push(&[1, 2]);
        assert!(!buf.is_empty());
        buf.clear();
        assert!(buf.is_empty());
    }
}
