use super::{RingBuffer, VisualizerBuffer};
use crate::utils::{MonoChannel, MonoChannelConsumer};
use std::ops::{Index, IndexMut};

#[derive(Clone)]
pub struct PeakBuffer {
    consumer: MonoChannelConsumer,
    buffer: RingBuffer<f32>,
    /// Maximum accumulator
    max_acc: f32,
    /// The gap between elements of the buffer in samples
    sample_delta: f32,
    /// Used to calculate the sample_delta
    sample_rate: f32,
    duration: f32,
    /// The current time, counts down from sample_delta to 0
    t: f32,
    /// The decay time for the peak amplitude to halve.
    decay: f32,
    /// This is set `set_sample_rate()` based on the sample_delta
    decay_weight: f32,
}

impl PeakBuffer {
    pub fn new(channel: MonoChannel, duration: f32, decay: f32) -> Self {
        let consumer = channel.get_consumer();
        Self {
            sample_rate: consumer.get_sample_rate(),
            consumer,
            buffer: RingBuffer::<f32>::new(1),
            max_acc: 0.,
            sample_delta: 0.,
            duration,
            t: 0.,
            decay,
            decay_weight: 0.0,
        }
    }

    pub fn set_decay(self: &mut Self, decay: f32) {
        self.decay = decay;
        self.update();
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
        ((sample_rate as f64 * duration as f64) / size as f64) as f32
    }

    fn decay_weight(decay: f32, size: usize, duration: f32) -> f32 {
        0.25f64.powf((decay as f64 / 1000. * (size as f64 / duration as f64)).recip()) as f32
    }

    fn update(self: &mut Self) {
        self.decay_weight = Self::decay_weight(self.decay, self.buffer.len(), self.duration);
        self.sample_delta = Self::sample_delta(self.buffer.len(), self.sample_rate, self.duration);
        self.t = self.sample_delta;
        self.buffer.clear();
    }
}

impl VisualizerBuffer<f32> for PeakBuffer {
    fn inner_buffer(&mut self) -> &mut RingBuffer<f32> {
        &mut self.buffer
    }

    fn consumer(&mut self) -> &mut MonoChannelConsumer {
        &mut self.consumer
    }

    fn enqueue(self: &mut Self, value: f32) {
        let value = value.abs();
        self.t -= 1.0;
        if self.t < 0.0 {
            let last_peak = self.buffer.peek();
            let peak = self.max_acc;

            // If the current peak is greater than the last one, we immediately enqueue it. If it's less than
            // the last one, we weigh the previous into the current one, analogous to how peak meters work.
            self.buffer.enqueue(if peak >= last_peak {
                peak
            } else {
                (last_peak * self.decay_weight) + (peak * (1.0 - self.decay_weight))
            });

            self.t += self.sample_delta;
            self.max_acc = 0.;
        }
        if value > self.max_acc {
            self.max_acc = value
        }
    }
}
