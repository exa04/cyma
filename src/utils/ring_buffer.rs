/// A fixed-size ring buffer that stores elements of type `T`.
///
/// Enqueues elements in a FIFO (First-In-First-Out) manner. Also provides
/// mutable and immutable iterators.
///
/// The `RingBuffer` allows enqueueing elements to the end of the buffer. When
/// an element is added to a full RingBuffer, the last element is removed from
/// the start of the ring buffer. Due to its fixed-size nature, the ring buffer
/// lives on the stack, is very fast and doesn't dynamically reallocate when an
/// element is added. It is foundational to visualizers, certain audio effects,
/// and really any use case where elements need to be sequentally enqueued in
/// this circular manner.
///
/// # Example
///
/// ```
/// use plext::utils::RingBuffer;
///
/// let mut buffer: RingBuffer<i32, 5> = RingBuffer::new();
///
/// // buffer = [0, 0, 0, 0, 0]
///
/// buffer.enqueue(1);
/// buffer.enqueue(2);
/// buffer.enqueue(3);
///
/// // buffer = [0, 0, 1, 2, 3]
///
/// assert_eq!(buffer.get(0), Some(&0));
/// assert_eq!(buffer.get(2), Some(&1));
///
/// buffer.enqueue(10);
/// buffer.enqueue(11);
/// buffer.enqueue(12);
///
/// // buffer = [2, 3, 10, 11, 12]
///
/// assert_eq!(buffer.get(1), Some(&3));
/// assert_eq!(buffer.get(3), Some(&11));
/// ```
#[derive(Debug, Clone)]

pub struct RingBuffer<T, const SIZE: usize> {
    head: usize,
    data: [T; SIZE],
}

impl<T: Default + Copy, const SIZE: usize> RingBuffer<T, SIZE> {
    pub fn new() -> Self {
        Self {
            head: 0,
            data: [T::default(); SIZE],
        }
    }

    /// Adds a new element of type `T` to the buffer. If the buffer is full, the
    /// oldest element is removed.
    pub fn enqueue(self: &mut Self, value: T) {
        self.data[self.head] = value;
        self.head = (self.head + 1) % SIZE;
    }

    /// Resets the buffer
    pub fn clear(self: &mut Self) {
        self.data.iter_mut().for_each(|x| *x = T::default());
    }

    /// Returns a reference to the element at `index` or `None` if out of bounds
    pub fn get(self: &Self, index: usize) -> Option<&T> {
        self.data.get((index + self.head) % SIZE)
    }

    /// Returns a mutable iterator to the buffer's data. If you want an
    /// immutable iterator, use [`into_iter()`](fn@into_iter).
    pub fn into_iter_mut(self: &mut Self) -> RingBufferIteratorMut<T, SIZE> {
        RingBufferIteratorMut {
            pos: self.head,
            ring_buffer: self,
        }
    }

    /// Returns an immutable iterator to the buffer's data. If you want a
    /// mutable iterator, use [`into_iter_mut()`](fn@into_iter_mut).
    ///
    /// # Example
    ///
    /// Using `into_iter()`, we can print a ring buffer in order.
    ///
    /// ```
    /// use plext::utils::RingBuffer;
    /// let mut buffer: RingBuffer<i32, 5> = RingBuffer::new();
    ///
    /// buffer.enqueue(1);
    /// buffer.enqueue(2);
    /// buffer.enqueue(3);
    ///
    /// buffer.into_iter().for_each(|n| {
    ///     print!("{} ", n);
    /// })
    /// ```
    pub fn into_iter(self: &Self) -> RingBufferIterator<T, SIZE> {
        RingBufferIterator {
            pos: self.head,
            ring_buffer: self,
        }
    }
}

pub struct RingBufferIterator<'a, T, const SIZE: usize> {
    pos: usize,
    ring_buffer: &'a RingBuffer<T, SIZE>,
}

impl<'a, T: Default + Copy, const SIZE: usize> Iterator for RingBufferIterator<'a, T, SIZE> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.pos += 1;
        self.pos %= SIZE;
        if self.pos != self.ring_buffer.head {
            return Some(self.ring_buffer.data[self.pos]);
        }
        None
    }
}

impl<'a, T: Default + Copy, const SIZE: usize> DoubleEndedIterator
    for RingBufferIterator<'a, T, SIZE>
{
    fn next_back(&mut self) -> Option<Self::Item> {
        self.pos = (self.pos + (SIZE - 1)) % SIZE;
        if self.pos != self.ring_buffer.head {
            return Some(self.ring_buffer.data[self.pos]);
        }
        None
    }
}

pub struct RingBufferIteratorMut<'a, T, const SIZE: usize> {
    pos: usize,
    ring_buffer: &'a mut RingBuffer<T, SIZE>,
}

impl<'a, T: Default + Copy, const SIZE: usize> Iterator for RingBufferIteratorMut<'a, T, SIZE> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.pos += 1;
        self.pos %= SIZE;
        if self.pos != self.ring_buffer.head {
            return Some(self.ring_buffer.data[self.pos]);
        }
        None
    }
}

/// A special type of ring buffer, intended for use in peak waveform analysis.
///
/// The `PeakWaveformRingBuffer` is a wrapper around the `RingBuffer` struct
/// that specifically handles waveforms. It stores elements of type `T` in
/// pairs, representing the minimum and maximum values of a waveform over a
/// certain duration. It provides methods for setting the sample rate and
/// duration, as well as enqueueing new values and retrieving the stored
/// waveform data.
///
/// For each pair `(T,T)` of samples that a `PeakWaveformRingBuffer<T>` holds,
/// The first element is the minimum (trough), and the second is the maximum
/// (crest) within the time frame that these peaks represent.
///
/// # Example
///
/// Here's how to create a 10-second long `PeakWaveformRingBuffer` with 512
/// samples at a sample rate of 44.1 kHz, stored as f32 values.
///
/// ```
/// use plext::utils::PeakWaveformRingBuffer;
/// let mut rb = PeakWaveformRingBuffer::<f32, 512>::new(44100., 10.0);
/// ```
///
/// It will now take `(44100*10)/512` enqueued samples for a new pair of maximum
/// and minimum values to be enqueued into the buffer.
pub struct PeakWaveformRingBuffer<T, const SIZE: usize> {
    ring_buffer: RingBuffer<(T, T), SIZE>,
    min_acc: T,
    max_acc: T,
    sample_rate: f32,
    duration: f32,
    sample_delta: f32,
    t: f32,
}

impl<const SIZE: usize> PeakWaveformRingBuffer<f32, SIZE> {
    /// Creates a new `PeakWaveformRingBuffer` with the specified sample rate
    /// and duration (in seconds)
    pub fn new(sample_rate: f32, duration: f32) -> Self {
        Self {
            ring_buffer: RingBuffer::<(f32, f32), SIZE>::new(),
            min_acc: 0.,
            max_acc: 0.,
            sample_delta: Self::sample_delta(sample_rate as f32, duration as f32),
            duration,
            sample_rate,
            t: 1.0,
        }
    }

    /// Sets the sample rate of the buffer and **clears** it
    pub fn set_sample_rate(self: &mut Self, sample_rate: f32) {
        self.sample_rate = sample_rate;
        self.sample_delta = Self::sample_delta(sample_rate, self.duration);
        self.ring_buffer.clear();
    }

    /// Sets the duration of the buffer (in seconds) and **clears** it
    pub fn set_duration(self: &mut Self, duration: f32) {
        self.duration = duration;
        self.sample_delta = Self::sample_delta(self.sample_rate, duration);
        self.ring_buffer.clear();
    }

    fn sample_delta(sample_rate: f32, duration: f32) -> f32 {
        (sample_rate * duration) / SIZE as f32
    }

    /// Adds a new element of type `T` to the buffer. If the buffer is full, the
    /// oldest element is removed.
    pub fn enqueue(self: &mut Self, value: f32) {
        self.t -= 1.0;
        if self.t <= 0.0 {
            self.ring_buffer.enqueue((self.min_acc, self.max_acc));
            self.t += self.sample_delta;
            self.min_acc = 0.;
            self.max_acc = 0.;
        }
        if value > self.max_acc {
            self.max_acc = value
        }
        if value < self.min_acc {
            self.min_acc = value
        }
    }

    /// Returns a reference to the element at `index` or `None` if out of bounds
    pub fn get(self: &Self, index: usize) -> Option<&(f32, f32)> {
        self.ring_buffer.get(index)
    }

    /// Returns an immutable iterator to the buffer's data. If you want a
    /// mutable iterator, use [`into_iter_mut()`](fn@into_iter_mut).
    pub fn into_iter(self: &Self) -> RingBufferIterator<(f32, f32), SIZE> {
        self.ring_buffer.into_iter()
    }

    /// Returns a mutable iterator to the buffer's data. If you want an
    /// immutable iterator, use [`into_iter()`](fn@into_iter).
    pub fn into_iter_mut(self: &mut Self) -> RingBufferIteratorMut<(f32, f32), SIZE> {
        self.ring_buffer.into_iter_mut()
    }
}

#[test]
fn test() {
    use rand::{rngs::OsRng, Rng};
    use std::time::Instant;

    const SAMPLE_RATE: usize = 44100;
    const BLOCK_SIZE: usize = 2048;
    const BLOCKS: usize = (SAMPLE_RATE * 1000) / BLOCK_SIZE;
    let signal: &[f32; 2048] = &{
        let mut x = [0.0; BLOCK_SIZE];
        for i in 0..BLOCK_SIZE {
            x[i] = OsRng.gen::<u32>() as f32;
        }
        x
    };

    let mut rb = PeakWaveformRingBuffer::<f32, 2048>::new(SAMPLE_RATE as f32, 1.0);

    let t = Instant::now();
    for _ in 0..BLOCKS {
        for x in signal {
            rb.enqueue(*x);
        }
    }
    dbg!(t.elapsed());
}
