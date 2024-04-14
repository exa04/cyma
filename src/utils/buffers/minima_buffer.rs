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
    min_acc: f32,
    sample_delta: f32,
    sample_rate: f32,
    duration: f32,
    t: f32,
}

impl MinimaBuffer {
    pub fn new(size: usize, sample_rate: f32, duration: f32) -> Self {
        let sample_delta = Self::sample_delta(size, sample_rate as f32, duration as f32);
        Self {
            buffer: RingBuffer::<f32>::new(size),
            min_acc: 0.,
            sample_delta,
            sample_rate,
            duration,
            t: sample_delta,
        }
    }

    pub fn set_sample_rate(self: &mut Self, sample_rate: f32) {
        self.sample_rate = sample_rate;
        self.sample_delta = Self::sample_delta(self.buffer.len(), sample_rate, self.duration);
        self.buffer.clear();
    }

    pub fn set_duration(self: &mut Self, duration: f32) {
        self.duration = duration;
        self.sample_delta = Self::sample_delta(self.buffer.len(), self.sample_rate, duration);
        self.buffer.clear();
    }

    fn sample_delta(size: usize, sample_rate: f32, duration: f32) -> f32 {
        (sample_rate * duration) / size as f32
    }
}

impl VisualizerBuffer<f32> for MinimaBuffer {
    fn enqueue(self: &mut Self, value: f32) {
        let value = value.abs();
        self.t -= 1.0;
        if self.t < 0.0 {
            self.buffer.enqueue(self.min_acc);
            self.t += self.sample_delta;
            self.min_acc = 0.;
        }
        if value < self.min_acc {
            self.min_acc = value
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
        self.sample_delta = Self::sample_delta(size, self.sample_rate, self.duration);
        self.buffer.clear();
    }

    /// Shrinks the buffer, **clearing it**.
    fn shrink(self: &mut Self, size: usize) {
        if self.buffer.len() == size {
            return;
        };
        self.buffer.shrink(size);
        self.sample_delta = Self::sample_delta(size, self.sample_rate, self.duration);
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
