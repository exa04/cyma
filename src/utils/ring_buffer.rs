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

    pub fn enqueue(self: &mut Self, value: T) {
        self.data[self.head] = value;
        self.head = (self.head + 1) % SIZE;
    }

    pub fn clear(self: &mut Self) {
        self.data.iter_mut().for_each(|x| *x = T::default());
    }

    pub fn into_iter(self: &mut Self) -> RingBufferIterator<T, SIZE> {
        RingBufferIterator {
            pos: self.head,
            ring_buffer: self,
        }
    }
}

pub struct PeakWaveformRingBuffer<T, const SIZE: usize> {
    pub ring_buffer: RingBuffer<(T, T), SIZE>,
    min_acc: T,
    max_acc: T,
    sample_rate: f32,
    duration: f32,
    sample_delta: f32,
    t: f32,
}

impl<const SIZE: usize> PeakWaveformRingBuffer<f32, SIZE> {
    pub fn new(sample_rate: f32, duration: f32) -> Self {
        Self {
            ring_buffer: RingBuffer::<(f32, f32), SIZE>::new(),
            min_acc: std::f32::INFINITY,
            max_acc: 0.,
            sample_delta: Self::sample_delta(sample_rate as f32, duration as f32),
            duration,
            sample_rate,
            t: 1.0,
        }
    }

    fn set_sample_rate(self: &mut Self, sample_rate: f32) {
        self.sample_rate = sample_rate;
        self.sample_delta = Self::sample_delta(sample_rate, self.duration);
        self.ring_buffer.clear();
    }

    fn set_duration(self: &mut Self, duration: f32) {
        self.duration = duration;
        self.sample_delta = Self::sample_delta(self.sample_rate, duration);
        self.ring_buffer.clear();
    }

    fn sample_delta(sample_rate: f32, duration: f32) -> f32 {
        (sample_rate * duration) / SIZE as f32
    }

    pub fn enqueue(self: &mut Self, value: f32) {
        self.t -= 1.0;
        if self.t <= 0.0 {
            self.ring_buffer.enqueue((self.min_acc, self.max_acc));
            self.t += self.sample_delta;
            self.min_acc = std::f32::INFINITY;
            self.max_acc = 0.;
        }
        if value > self.max_acc {
            self.max_acc = value
        }
        if value < self.min_acc {
            self.min_acc = value
        }
    }
}

pub struct RingBufferIterator<'a, T, const SIZE: usize> {
    pos: usize,
    ring_buffer: &'a mut RingBuffer<T, SIZE>,
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
