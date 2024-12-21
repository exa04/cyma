use crate::utils::RingBuffer;

pub trait Accumulator {
    fn accumulate(&mut self, sample: f32) -> Option<f32>;
    fn prev(&self) -> f32;
    fn set_sample_rate(&mut self, sample_rate: f32);
    fn set_size(&mut self, size: usize);
}

#[inline]
pub fn sample_delta(size: usize, sample_rate: f32, duration: f32) -> f32 {
    ((sample_rate as f64 * duration as f64) / size as f64) as f32
}

#[inline]
pub fn decay_weight(decay: f32, size: usize, duration: f32) -> f32 {
    0.25f64.powf((decay as f64 / 1000. * (size as f64 / duration as f64)).recip()) as f32
}

pub struct PeakAccumulator {
    /// Maximum accumulator
    max_acc: f32,
    /// Previous accumulator value
    prev: f32,
    size: usize,
    duration: f32,
    decay: f32,
    sample_rate: f32,
    /// The current time, counts down from sample_delta to 0
    t: f32,
    /// The decay time for the peak amplitude to halve.
    sample_delta: f32,
    decay_weight: f32,
}

impl PeakAccumulator {
    pub fn new(duration: f32, decay: f32) -> Self {
        Self {
            duration,
            decay,
            max_acc: 0.0,
            prev: 0.0,
            size: 1,
            sample_delta: 1.0,
            sample_rate: 1.0,
            t: 0.0,
            decay_weight: 0.0,
        }
    }

    fn update(self: &mut Self) {
        self.decay_weight = decay_weight(self.decay, self.size, self.duration);
        self.sample_delta = sample_delta(self.size, self.sample_rate, self.duration);
        self.t = 0.0;
    }
}

impl Accumulator for PeakAccumulator {
    #[inline]
    fn accumulate(&mut self, sample: f32) -> Option<f32> {
        self.max_acc = self.max_acc.max(sample.abs());
        self.t += 1.0;

        if self.t > self.sample_delta {
            let peak = self.max_acc;

            self.t -= self.sample_delta;
            self.max_acc = 0.;

            let next = if peak >= self.prev {
                peak
            } else {
                self.prev * self.decay_weight + peak * (1.0 - self.decay_weight)
            };

            self.prev = next;

            Some(next)
        } else {
            None
        }
    }

    #[inline]
    fn prev(&self) -> f32 {
        self.prev
    }

    #[inline]
    fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
        self.update();
    }

    #[inline]
    fn set_size(&mut self, size: usize) {
        self.size = size;
        self.update();
    }
}

pub struct MinimumAccumulator {
    /// Maximum accumulator
    min_acc: f32,
    /// Previous accumulator value
    prev: f32,
    size: usize,
    duration: f32,
    decay: f32,
    sample_rate: f32,
    /// The current time, counts down from sample_delta to 0
    t: f32,
    /// The decay time for the minimum amplitude to halve.
    sample_delta: f32,
    decay_weight: f32,
}

impl MinimumAccumulator {
    pub fn new(duration: f32, decay: f32) -> Self {
        Self {
            duration,
            decay,
            min_acc: 0.0,
            prev: 0.0,
            size: 1,
            sample_delta: 1.0,
            sample_rate: 1.0,
            t: 0.0,
            decay_weight: 0.0,
        }
    }

    fn update(self: &mut Self) {
        self.decay_weight = decay_weight(self.decay, self.size, self.duration);
        self.sample_delta = sample_delta(self.size, self.sample_rate, self.duration);
        self.t = 0.0;
    }
}

impl Accumulator for MinimumAccumulator {
    #[inline]
    fn accumulate(&mut self, sample: f32) -> Option<f32> {
        self.min_acc = self.min_acc.min(sample.abs());
        self.t += 1.0;

        if self.t > self.sample_delta {
            let minimum = self.min_acc;

            self.t -= self.sample_delta;
            self.min_acc = 0.;

            let next = if minimum >= self.prev {
                minimum
            } else {
                self.prev * self.decay_weight + minimum * (1.0 - self.decay_weight)
            };

            self.prev = next;

            Some(next)
        } else {
            None
        }
    }

    #[inline]
    fn prev(&self) -> f32 {
        self.prev
    }

    #[inline]
    fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
        self.update();
    }

    #[inline]
    fn set_size(&mut self, size: usize) {
        self.size = size;
        self.update();
    }
}

pub struct RMSAccumulator {
    duration: f32,
    rms_window: f32,
    prev: f32,

    size: usize,
    sample_rate: f32,
    t: f32,
    sum_acc: f32,
    sample_delta: f32,
    squared_buffer: RingBuffer<f32>,
}

impl RMSAccumulator {
    pub fn new(duration: f32, rms_window: f32) -> Self {
        Self {
            duration,
            rms_window,
            prev: 0.0,

            size: 1,
            sample_delta: 0.0,
            t: 0.0,
            sum_acc: 0.0,
            sample_rate: 0.0,
            squared_buffer: RingBuffer::<f32>::new(0),
        }
    }

    fn update(self: &mut Self) {
        self.sample_delta = sample_delta(self.size, self.sample_rate, self.duration);

        let rms_size = (self.sample_rate as f64 * (self.rms_window as f64 / 1000.0)) as usize;
        self.squared_buffer.resize(rms_size);
        self.t = 0.0;
    }
}

impl Accumulator for RMSAccumulator {
    #[inline]
    fn accumulate(&mut self, sample: f32) -> Option<f32> {
        let squared_value = sample * sample;

        self.sum_acc -= self.squared_buffer.tail();
        self.squared_buffer.enqueue(squared_value);
        self.sum_acc += squared_value;

        self.t -= 1.0;

        if self.t <= 0.0 {
            let rms = (self.sum_acc / self.squared_buffer.len() as f32).sqrt();
            self.t += self.sample_delta;

            let value = if rms.is_nan() { 0.0 } else { rms };

            self.prev = value;

            Some(value)
        } else {
            None
        }
    }

    #[inline]
    fn prev(&self) -> f32 {
        self.prev
    }

    #[inline]
    fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
        self.update();
    }

    #[inline]
    fn set_size(&mut self, size: usize) {
        self.size = size;
        self.update();
    }
}
