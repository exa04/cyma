use std::ops::{Index, IndexMut};

use super::{RingBuffer, VisualizerBuffer};

/// Analogous to the `PeakBuffer`, save for the fact that it stores the minimum
/// absolute values instead of the maximum absolute values of a signal over time.
///
/// This buffer is useful for gain reduction meters / graphs. For regular
/// visualizers that are meant to display peak information, such as peak graphs, do
/// use a `PeakBuffer`.
#[derive(Clone, Default)]
pub struct MinimaBuffer {
    buffer: RingBuffer<f32>,
    // Minimum and maximum accumulators
    max_acc: f32,
    // The gap between elements of the buffer in samples
    sample_delta: f32,
    // Used to calculate the sample_delta
    sample_rate: f32,
    duration: f32,
    // The current time, counts down from sample_delta to 0
    t: f32,
    /// The decay time for the peak amplitude to halve.
    decay: f32,
    // This is set `set_sample_rate()` based on the sample_delta
    decay_weight: f32,
}

impl MinimaBuffer {
    pub fn new(size: usize, duration: f32, decay: f32) -> Self {
        let decay_weight = Self::decay_weight(decay, size, duration);
        Self {
            buffer: RingBuffer::<f32>::new(size),
            max_acc: 0.,
            sample_delta: 0.,
            sample_rate: 0.,
            duration,
            t: 0.,
            decay,
            decay_weight,
        }
    }

    pub fn set_decay(self: &mut Self, decay: f32) {
        self.decay = decay;
        self.update();
    }

    pub fn set_sample_rate(self: &mut Self, sample_rate: f32) {
        self.sample_rate = sample_rate;
        self.update();
        self.buffer.clear();
    }

    pub fn set_duration(self: &mut Self, duration: f32) {
        self.duration = duration;
        self.update();
        self.buffer.clear();
    }

    fn sample_delta(size: usize, sample_rate: f32, duration: f32) -> f32 {
        ((sample_rate as f64 * duration as f64) / size as f64) as f32
    }

    fn decay_weight(decay: f32, size: usize, duration: f32) -> f32 {
        0.25f64.powf((decay as f64 / 1000. * (size as f64 / duration as f64)).recip()) as f32
    }

    fn update(self: &mut Self) {
        self.decay_weight = Self::decay_weight(self.decay, self.buffer.len(), self.duration);
        self.sample_delta = Self::sample_delta(self.buffer.len(), self.sample_rate, self.duration);
        self.t = self.sample_delta;
    }
}

impl VisualizerBuffer<f32> for MinimaBuffer {
    fn enqueue(self: &mut Self, value: f32) {
        let value = value.abs();
        self.t -= 1.0;
        if self.t < 0.0 {
            let last_peak = self.buffer.peek();
            let mut peak = self.max_acc;

            // If the current peak is less than the last one, we immediately enqueue it. If it's greater than
            // the last one, we weigh the previous into the current one, analogous to how peak meters work.
            self.buffer.enqueue(if peak <= last_peak {
                peak
            } else {
                (last_peak * self.decay_weight) + (peak * (1.0 - self.decay_weight))
            });

            self.t += self.sample_delta;
            self.max_acc = 0.;
        }
        if value < self.max_acc {
            self.max_acc = value
        }
    }

    fn enqueue_buffer(
        self: &mut Self,
        buffer: &mut nih_plug::buffer::Buffer,
        channel: Option<usize>,
    ) {
        match channel {
            Some(channel) => {
                for sample in buffer.as_slice()[channel].into_iter() {
                    self.enqueue(*sample);
                }
            }
            None => {
                for sample in buffer.iter_samples() {
                    self.enqueue(
                        (1. / (&sample).len() as f32) * sample.into_iter().map(|x| *x).sum::<f32>(),
                    );
                }
            }
        }
    }

    fn len(&self) -> usize {
        self.buffer.len()
    }

    fn clear(self: &mut Self) {
        self.buffer.clear();
    }

    /// Grows the buffer, **clearing it**.
    fn grow(self: &mut Self, size: usize) {
        if self.buffer.len() == size {
            return;
        };
        self.buffer.grow(size);
        self.update();
        self.buffer.clear();
    }

    /// Shrinks the buffer, **clearing it**.
    fn shrink(self: &mut Self, size: usize) {
        if self.buffer.len() == size {
            return;
        };
        self.buffer.shrink(size);
        self.update();
        self.buffer.clear();
    }
}

impl Index<usize> for MinimaBuffer {
    type Output = f32;

    fn index(&self, index: usize) -> &Self::Output {
        self.buffer.index(index)
    }
}
impl IndexMut<usize> for MinimaBuffer {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.buffer.index_mut(index)
    }
}
