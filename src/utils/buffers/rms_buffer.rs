use nih_plug::buffer::Buffer;
use std::ops::{Index, IndexMut};

use crate::utils::{MonoChannel, MonoChannelConsumer};

use super::{RingBuffer, VisualizerBuffer};

/// Stores RMS amplitudes over time.
///
/// This buffer keeps track of the windowed root mean squared amplitudes of a
/// signal.
///
/// It needs to be provided a sample rate after initialization - do this inside your
/// [`initialize()`](nih_plug::plugin::Plugin::initialize)` function!
#[derive(Clone)]
pub struct RMSBuffer {
    consumer: MonoChannelConsumer,
    buffer: RingBuffer<f32>,
    /// The duration of RMS values that the buffer captures, in s (example: 10.0)
    duration: f32,
    /// The time window in which the RMS is calculated, in ms (example: 300.0)
    rms_duration: f32,

    /// The sample rate (example: 44100.0)
    sample_rate: f32,
    /// The current time
    t: f32,
    /// The squared sum accumulator - When a sample gets enqueued, its squared value
    /// is added into this. When it gets removed, its squared value is removed from
    /// here.
    sum_acc: f32,
    /// The time it takes (in samples) for an RMS value to get enqueued
    sample_delta: f32,
    /// The buffer of squared sums - This is needed so that the squared samples can
    /// be removed from the `sum_acc`
    squared_buffer: RingBuffer<f32>,
}

impl RMSBuffer {
    /// Creates a new RMSBuffer
    ///
    /// * `size` - The length of the buffer in samples
    /// * `duration` - The duration (in seconds) of the RMS data inside the buffer, in seconds
    /// * `rms_duration` - The duration of each RMS window, in milliseconds
    pub fn new(channel: MonoChannel, duration: f32, rms_duration: f32) -> Self {
        let consumer = channel.get_consumer();
        Self {
            sample_rate: consumer.get_sample_rate(),
            consumer,
            buffer: RingBuffer::<f32>::new(1),
            duration,
            rms_duration,

            t: 0.0,
            sum_acc: 0.0,
            sample_delta: 0.0,
            squared_buffer: RingBuffer::<f32>::new(0),
        }
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
        self.update();
    }

    fn update(&mut self) {
        self.sample_delta =
            ((self.sample_rate as f64 * self.duration as f64) / self.buffer.len() as f64) as f32;

        let rms_size = (self.sample_rate as f64 * (self.rms_duration as f64 / 1000.0)) as usize;
        self.squared_buffer.resize(rms_size);

        self.clear();
    }
}

impl Index<usize> for RMSBuffer {
    type Output = f32;

    fn index(&self, index: usize) -> &Self::Output {
        &self.buffer[index]
    }
}

impl IndexMut<usize> for RMSBuffer {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.buffer[index]
    }
}

impl VisualizerBuffer<f32> for RMSBuffer {
    fn enqueue(self: &mut Self, value: f32) {
        let squared_value = value * value;

        self.sum_acc -= self.squared_buffer.tail();
        self.squared_buffer.enqueue(squared_value);
        self.sum_acc += squared_value;

        self.t -= 1.0;

        if self.t <= 0.0 {
            let rms = (self.sum_acc / self.squared_buffer.len() as f32).sqrt();
            if rms.is_nan() {
                self.buffer.enqueue(0.0);
            } else {
                self.buffer.enqueue(rms);
            }
            self.t += self.sample_delta
        }
    }

    fn enqueue_buffer(self: &mut Self, buffer: &mut Buffer, channel: Option<usize>) {
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

    fn clear(self: &mut Self) {
        self.sum_acc = 0.0;
        self.t = self.sample_delta;
        self.buffer.clear();
        self.squared_buffer.clear();
    }

    fn grow(self: &mut Self, size: usize) {
        self.clear();
        self.buffer.grow(size);
    }

    fn shrink(self: &mut Self, size: usize) {
        self.clear();
        self.buffer.shrink(size);
    }

    fn len(self: &Self) -> usize {
        self.buffer.len()
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
}
