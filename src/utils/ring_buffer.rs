#[derive(Debug)]
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

// impl IntoIterator for RingBuffer {
//     type Item = f32;

//     type IntoIter = RingBufferIntoIterator;

//     fn into_iter(self) -> Self::IntoIter {
//         RingBufferIntoIterator {
//             ring_buffer: self,
//             index: 0,
//         }
//     }
// }

// pub struct RingBufferIntoIterator {
//     ring_buffer: RingBuffer,
//     index: usize,
// }

// impl Iterator for RingBufferIntoIterator {
//     type Item = f32;
//     fn next(&mut self) -> Option<f32> {
//         if self.index >= self.ring_buffer.size {
//             return None;
//         }
//         self.index += 1;
//         self.ring_buffer
//             .data
//             .get((self.index + self.ring_buffer.head - 1) % self.ring_buffer.size)
//             .copied()
//     }
// }
