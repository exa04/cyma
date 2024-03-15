use nih_plug::buffer::Buffer;
use num_traits::real::Real;
use std::fmt::Debug;

use super::{ring_buffer::Iter, RingBuffer};

#[derive(Clone, PartialEq, Default)]
pub struct PeakBuffer<T> {
    buffer: RingBuffer<(T, T)>,
    // Minimum and maximum accumulators
    min_acc: T,
    max_acc: T,
    // The gap between elements of the buffer in samples
    sample_delta: f32,
    // Used to calculate the sample_delta
    sample_rate: f32,
    duration: f32,
    // The current time, counts down from sample_delta to 0
    t: f32,
}

impl<T> PeakBuffer<T>
where
    T: Clone + Copy + Default + Debug + PartialOrd + Real,
{
    pub fn new(size: usize, sample_rate: f32, duration: f32) -> Self {
        let sample_delta = Self::sample_delta(size, sample_rate as f32, duration as f32);
        Self {
            buffer: RingBuffer::<(T, T)>::new(size),
            min_acc: T::max_value(),
            max_acc: T::default(),
            sample_delta,
            sample_rate,
            duration,
            t: sample_delta,
        }
    }

    pub fn set_size(self: &mut Self, size: usize) {
        if self.buffer.len() == size {
            return;
        };
        self.buffer.resize(size);
        self.sample_delta = Self::sample_delta(size, self.sample_rate, self.duration);
        self.buffer.clear();
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

    pub fn enqueue(self: &mut Self, value: T) {
        let value = value.abs();
        self.t -= 1.0;
        if self.t < 0.0 {
            self.buffer.enqueue((self.min_acc, self.max_acc));
            self.t += self.sample_delta;
            self.min_acc = T::max_value();
            self.max_acc = T::default();
        }
        if value > self.max_acc {
            self.max_acc = value
        }
        if value < self.min_acc {
            self.min_acc = value
        }
    }

    pub fn len(&self) -> usize {
        self.buffer.len()
    }
}

impl PeakBuffer<f32> {
    /// Enqueues an entire [`Buffer`], mono-summing it if necessary.
    pub fn enqueue_buffer(self: &mut Self, buffer: &mut Buffer) {
        for sample in buffer.iter_samples() {
            self.enqueue(
                (1. / (&sample).len() as f32) * sample.into_iter().map(|x| *x).sum::<f32>(),
            );
        }
    }
}

impl<'a, T: Copy> IntoIterator for &'a PeakBuffer<T> {
    type Item = &'a (T, T);
    type IntoIter = Iter<'a, (T, T)>;

    /// Creates an iterator from a reference.
    fn into_iter(self) -> Self::IntoIter {
        (&self.buffer).into_iter()
    }
}

#[cfg(test)]
mod tests {
    use super::PeakBuffer;

    #[test]
    fn enqueue() {
        let mut rb = PeakBuffer::<f32>::new(16, 4.0, 8.0);

        rb.enqueue(2.);
        rb.enqueue(9.);
        rb.enqueue(19.);
        rb.enqueue(-10.);
        rb.enqueue(4.);
        rb.enqueue(6.);

        let buffer = rb.buffer.to_vec();

        assert_eq!(buffer[0].0, 2.);
        assert_eq!(buffer[0].1, 9.);
        assert_eq!(buffer[1].0, 10.);
        assert_eq!(buffer[1].1, 19.);
    }
}
