use std::iter;

#[derive(Debug)]
pub struct HeapRingBuffer {
    head: usize,
    pub size: usize,
    data: Vec<f32>,
}

impl HeapRingBuffer {
    pub fn new(size: usize) -> Self {
        Self {
            head: 0,
            size,
            data: iter::repeat(0.0 as f32).take(size).collect(),
        }
    }

    pub fn enqueue(self: &mut Self, value: f32) {
        self.data[self.head] = value;
        self.head = (self.head + 1) % self.size;
    }

    pub fn clear(self: &mut Self) {
        self.data.iter_mut().for_each(|x| *x = 0.0);
    }
}
