use crate::utils::ring_buffer::RingBuffer;

use super::VisualizerBuffer;
use crate::utils::OutletConsumer;
use std::ops::{Index, IndexMut};
use std::sync::Arc;

/// A special type of ring buffer for waveform analysis.
///
/// This is a wrapper around the [`RingBuffer`](crate::utils::RingBuffer) struct
/// that handles waveforms. It stores elements of type T in pairs, to represent the
/// minimum and maximum values of a waveform over a certain interval.
///
/// For each pair `(T,T)` of samples that a WaveformBuffer holds, the first element
/// is the local minimum, and the second is the local maximum within the respective
/// time frame.
///
/// These values can be used to construct a zoomed-out representation of the audio
/// data without losing peak information - which is why this buffer is used in the
/// [`Oscilloscope`](crate::editor::views::Oscilloscope).
#[derive(Clone)]
pub struct WaveformBuffer {
    consumer: Arc<dyn OutletConsumer>,
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
    pub fn new(consumer: impl OutletConsumer + 'static, duration: f32) -> Self {
        Self {
            consumer: Arc::new(consumer),
            buffer: RingBuffer::<(f32, f32)>::new(1),
            min_acc: f32::MAX,
            max_acc: f32::MIN,
            sample_delta: 0.,
            sample_rate: 0.,
            duration,
            t: 0.,
        }
    }

    pub fn set_sample_rate(self: &mut Self, sample_rate: f32) {
        self.sample_rate = sample_rate;
        self.update();
    }

    pub fn set_duration(self: &mut Self, duration: f32) {
        self.duration = duration;
        self.update();
    }

    fn sample_delta(size: usize, sample_rate: f32, duration: f32) -> f32 {
        (sample_rate * duration) / size as f32
    }

    fn update(self: &mut Self) {
        self.sample_delta = Self::sample_delta(self.buffer.len(), self.sample_rate, self.duration);
        self.t = self.sample_delta;
        self.buffer.clear();
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

    fn enqueue_latest(&mut self) {
        let sample_rate = self.consumer.get_sample_rate();

        if sample_rate != self.sample_rate {
            self.set_sample_rate(sample_rate);
        }

        self.consumer.receive().iter().for_each(|sample| {
            self.enqueue(*sample);
        });
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
