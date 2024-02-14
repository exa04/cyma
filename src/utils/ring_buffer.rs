#[derive(Debug)]

/// A generic ring buffer
pub struct RingBuffer<T, const BUFFER_SIZE: usize> {
    head: usize,
    pub size: usize,
    pub data: [T; BUFFER_SIZE],
}

impl<T: Default + Copy, const BUFFER_SIZE: usize> RingBuffer<T, BUFFER_SIZE> {
    pub fn new() -> Self {
        Self {
            head: 0,
            size: BUFFER_SIZE,
            data: [T::default(); BUFFER_SIZE],
        }
    }

    pub fn enqueue(self: &mut Self, value: T) {
        self.data[self.head] = value;
        self.head = (self.head + 1) % self.size;
    }

    pub fn clear(self: &mut Self) {
        self.data.iter_mut().for_each(|x| *x = T::default());
    }
}

/// A ring buffer which implements higher-quality downsampling by averaging
/// incoming samples before inserting them.
pub struct DownsampledRingBuffer<const BUFFER_SIZE: usize> {
    /// The downsampling ratio - For each `sample_delta`th input sample, the sum
    /// of previously accumulated sampels goes into the DownsampledRingBuffer
    sample_delta: f32,
    pub ring_buffer: RingBuffer<f32, BUFFER_SIZE>,
    /// Keeps track of current "time"
    t: f32,
    /// Averages samples before inserting them
    accumulator: f32,
}

impl<const BUFFER_SIZE: usize> DownsampledRingBuffer<BUFFER_SIZE> {
    /// Creates a new instance of `DownsampledRingBuffer` with the specified
    /// sample rate and length.
    ///
    /// # Arguments
    ///
    /// * `sample_rate` - The sample rate in Hz.
    /// * `length` - The length of the buffer in seconds. Higher values
    ///   correspond to higher downsampling ratios
    ///
    /// # Returns
    ///
    /// A new instance of `DownsampledRingBuffer`.
    pub fn new(sample_rate: f32, length: f32) -> Self {
        Self {
            ring_buffer: RingBuffer::new(),
            sample_delta: ((sample_rate / BUFFER_SIZE as f32) * length),
            t: 0.,
            accumulator: 0.0,
        }
    }

    /// Enqueues a new sample by adding it to the accumulator first and pushing
    /// it to the ring buffer if the gap between the last pushed sample is large
    /// enough.
    pub fn enqueue(self: &mut Self, value: f32) {
        // Add new sample to the accumulator
        self.accumulator += value;

        // `sample_delta` is the gap between input samples for which we add a
        // new sample to the downsampled buffer. So when our steadily-increasing
        // `t` is above the delta, it's time to enqueue the accumulated samples.
        if self.t >= self.sample_delta {
            self.accumulator /= self.t;
            self.ring_buffer.enqueue(self.accumulator);

            self.t -= self.sample_delta;
            self.accumulator = 0.0;
        }
        self.t += 1.;
    }
}
