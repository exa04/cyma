use crate::utils::ring_buffer::RingBuffer;

use std::ops::{Index, IndexMut};

use super::VisualizerBuffer;

/// A special type of ring buffer, intended for use in peak waveform analysis.
///
/// This is a wrapper around the [`RingBuffer`](crate::utils::RingBuffer) struct
/// that specifically handles waveforms. It stores elements of type T in pairs
/// to represent the minimum and maximum values of a waveform over a certain
/// interval. It provides methods for setting the sample rate and duration, as
/// well as enqueueing new values and retrieving the stored waveform data.
///
/// For each pair `(T,T)` of samples that a WaveformBuffer holds, the first
/// element is the local minimum, and the second is the local maximum within the
/// respective time frame.
///
/// ![Alt version](http://127.0.0.1:5500/img.svg)
///
/// These values can be used to construct a zoomed-out representation of the
/// audio data without losing peak information - which is why this buffer is
/// used in the [`Oscilloscope`](crate::editor::views::Oscilloscope).
#[derive(Clone, PartialEq, Default)]
pub struct WaveformBuffer {
    buffer: RingBuffer<(f32, f32)>,
    // Minimum and maximum accumulators
    min_acc: f32,
    max_acc: f32,
    // The gap between elements of the buffer in samples
    sample_delta: f32,
    // Used to calculate the sample_delta
    sample_rate: f32,
    duration: f32,
    // The current time, counts down from sample_delta to 0
    t: f32,
}

impl WaveformBuffer {
    /// Creates a new `WaveformBuffer` with the specified sample rate and
    /// duration (in seconds).
    pub fn new(size: usize, sample_rate: f32, duration: f32) -> Self {
        let sample_delta = Self::sample_delta(size, sample_rate as f32, duration as f32);
        Self {
            buffer: RingBuffer::<(f32, f32)>::new(size),
            min_acc: f32::MAX,
            max_acc: f32::MIN,
            sample_delta,
            sample_rate,
            duration,
            t: sample_delta,
        }
    }
    /// Sets the sample rate of the buffer and **clears** it.
    pub fn set_sample_rate(self: &mut Self, sample_rate: f32) {
        self.sample_rate = sample_rate;
        self.sample_delta = Self::sample_delta(self.buffer.len(), sample_rate, self.duration);
        self.buffer.clear();
    }

    /// Sets the duration of the buffer (in seconds) and **clears** it.
    pub fn set_duration(self: &mut Self, duration: f32) {
        self.duration = duration;
        self.sample_delta = Self::sample_delta(self.buffer.len(), self.sample_rate, duration);
        self.buffer.clear();
    }

    fn sample_delta(size: usize, sample_rate: f32, duration: f32) -> f32 {
        (sample_rate * duration) / size as f32
    }
}

impl VisualizerBuffer<f32> for WaveformBuffer {
    fn enqueue(self: &mut Self, value: f32) {
        self.t -= 1.0;
        if self.t < 0.0 {
            self.buffer.enqueue((self.min_acc, self.max_acc));
            self.t += self.sample_delta;
            self.min_acc = f32::MAX;
            self.max_acc = f32::MIN;
        }
        if value > self.max_acc {
            self.max_acc = value
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

    fn grow(self: &mut Self, size: usize) {
        if size == self.buffer.len() {
            return;
        }
        self.buffer.grow(size);
        self.sample_delta = Self::sample_delta(size, self.sample_rate, self.duration);
        self.buffer.clear();
    }

    fn shrink(self: &mut Self, size: usize) {
        if size == self.buffer.len() {
            return;
        }
        self.buffer.shrink(size);
        self.sample_delta = Self::sample_delta(size, self.sample_rate, self.duration);
        self.buffer.clear();
    }
}

impl Index<usize> for WaveformBuffer {
    type Output = (f32, f32);

    fn index(&self, index: usize) -> &Self::Output {
        self.buffer.index(index)
    }
}
impl IndexMut<usize> for WaveformBuffer {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.buffer.index_mut(index)
    }
}
