use std::fmt::Debug;

use crate::utils::ring_buffer::{Iter, RingBuffer};

use num_traits::real::Real;

/// A special type of ring buffer, intended for use in peak waveform analysis.
///
/// This is a wrapper around the [`RingBuffer`](crate::utils::RingBuffer) struct
/// that specifically handles waveforms. It stores elements of type T in pairs
/// to represent the minimum and maximum values of a waveform over a certain
/// interval. It provides methods for setting the sample rate and duration, as
/// well as enqueueing new values and retrieving the stored waveform data.
///
/// For each pair `(T,T)` of samples that a PeakRingBuffer holds, the first
/// element is the local minimum, and the second is the local maximum within the
/// respective time frame.
///
/// ![Alt version](http://127.0.0.1:5500/img.svg)
///
/// These values can be used to construct a zoomed-out representation of the
/// audio data without losing peak information - which is why this buffer is
/// used in the [`Oscilloscope`](crate::editor::views::Oscilloscope).
///
/// # Example
///
/// Here's how to create a `PeakRingBuffer` with 512 samples, stored as f32
/// values. We'll provide a sample rate of 44.1 kHz and a length of 10 seconds.
///
/// ```
/// use plext::utils::PeakRingBuffer;
/// let mut rb = PeakRingBuffer::<f32>::new(512, 10.0, 44100.);
/// ```
///
/// When we later push into this buffer, it will accumulate samples according to
/// these restrictions. It will take (44100*10)/512 enqueued samples for a new
/// pair of maximum and minimum values to be added to the buffer.
#[derive(Clone, PartialEq, Default)]
pub struct PeakRingBuffer<T> {
    buffer: RingBuffer<(T, T)>,
    min_acc: T,
    max_acc: T,
    sample_rate: f32,
    duration: f32,
    sample_delta: f32,
    t: f32,
}

impl<T> PeakRingBuffer<T>
where
    T: Clone + Copy + Default + Debug + PartialOrd + Real,
{
    /// Creates a new `PeakRingBuffer` with the specified sample rate
    /// and duration (in seconds)
    pub fn new(size: usize, sample_rate: f32, duration: f32) -> Self {
        Self {
            buffer: RingBuffer::<(T, T)>::new(size),
            min_acc: T::default(),
            max_acc: T::default(),
            sample_delta: Self::sample_delta(size, sample_rate as f32, duration as f32),
            duration,
            sample_rate,
            t: 1.0,
        }
    }

    /// Sets the size of the buffer and **clears** it
    pub fn set_size(self: &mut Self, size: usize) {
        if self.buffer.len() == size {
            return;
        };
        self.buffer.resize(size);
        self.sample_delta = Self::sample_delta(size, self.sample_rate, self.duration);
        self.buffer.clear();
    }

    /// Sets the sample rate of the buffer and **clears** it
    pub fn set_sample_rate(self: &mut Self, sample_rate: f32) {
        self.sample_rate = sample_rate;
        self.sample_delta = Self::sample_delta(self.buffer.len(), sample_rate, self.duration);
        self.buffer.clear();
    }

    /// Sets the duration of the buffer (in seconds) and **clears** it
    pub fn set_duration(self: &mut Self, duration: f32) {
        self.duration = duration;
        self.sample_delta = Self::sample_delta(self.buffer.len(), self.sample_rate, duration);
        self.buffer.clear();
    }

    fn sample_delta(size: usize, sample_rate: f32, duration: f32) -> f32 {
        (sample_rate * duration) / size as f32
    }

    /// Adds a new element of type `T` to the buffer.
    ///
    /// If the buffer is full, the oldest element is removed.
    pub fn enqueue(self: &mut Self, value: T) {
        self.t -= 1.0;
        if self.t <= 0.0 {
            self.buffer.enqueue((self.min_acc, self.max_acc));
            self.t += self.sample_delta;
            self.min_acc = T::max_value();
            self.max_acc = T::min_value();
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

impl<'a, T: Copy> IntoIterator for &'a PeakRingBuffer<T> {
    type Item = &'a (T, T);
    type IntoIter = Iter<'a, (T, T)>;

    /// Creates an iterator from a reference.
    fn into_iter(self) -> Self::IntoIter {
        (&self.buffer).into_iter()
    }
}
