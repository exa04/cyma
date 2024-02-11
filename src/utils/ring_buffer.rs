use std::iter;

/// A generic ring buffer
///
/// Allows you to store values in a circular structure where, in a
/// First-In-First-Out-fashion, values can be appended without the need for
/// resizing or any reallocation.
///
/// # Example
///
/// ```
/// use plext::RingBuffer;
/// let mut rb = RingBuffer::<f32>::new(2);
/// rb.enqueue(1.0);
/// rb.enqueue(2.0);
/// rb.enqueue(3.0);
/// // [2.0, 3.0]
/// rb.resize_and_copy(4);
/// // [0.0, 0.0, 2.0, 3.0]
/// rb.enqueue(4.0);
/// // [0.0, 2.0, 3.0, 4.0]
/// rb.resize(2);
/// // [0.0, 0.0]
/// ```
#[derive(Debug, Clone)]
pub struct RingBuffer<T> {
    head: usize,
    size: usize,
    data: Vec<T>,
}

impl<T: Clone + Copy + Default + std::fmt::Debug> RingBuffer<T> {
    /// Creates a new `RingBuffer` of a given size and fills it with the default
    /// values for the given data type.
    ///
    /// # Example
    ///
    /// ```
    /// use plext::utils::RingBuffer;
    /// let mut rb = RingBuffer::<f32>::new(4);
    /// // [0.0, 0.0, 0.0, 0.0]
    /// ```
    pub fn new(size: usize) -> Self {
        return Self {
            head: 0,
            size,
            data: iter::repeat(T::default()).take(size).collect(),
        };
    }

    /// Enqueues an element into the `RingBuffer`
    ///
    /// # Example
    ///
    /// ```
    /// use plext::utils::RingBuffer;
    /// let mut rb = RingBuffer::<f32>::new(3);
    /// rb.enqueue(1.0);
    /// // [0.0, 0.0, 1.0]
    /// rb.enqueue(2.0);
    /// // [0.0, 1.0, 2.0]
    /// rb.enqueue(3.0);
    /// // [1.0, 2.0, 3.0]
    /// rb.enqueue(4.0);
    /// // [2.0, 3.0, 4.0]
    /// ```
    pub fn enqueue(self: &mut Self, value: T) {
        self.data[self.head] = value;
        self.head = (self.head + 1) % self.size;
    }

    /// Resizes the `RingBuffer`. This **empties** the buffer. If you want to
    /// copy the previous values into the new buffer, use `resize_and_copy()`.
    ///
    /// # Example
    ///
    /// ```
    /// use plext::utils::RingBuffer;
    /// let mut rb = RingBuffer::<f32>::new(2);
    /// rb.enqueue(1.0);
    /// rb.enqueue(2.0);
    /// rb.enqueue(3.0);
    /// // [2.0, 3.0]
    /// rb.resize(4);
    /// // [0.0, 0.0, 0.0, 0.0]
    /// rb.enqueue(4.0);
    /// // [0.0, 0.0, 0.0, 4.0]
    /// rb.resize(2);
    /// // [0.0, 4.0]
    /// ```
    pub fn resize(self: &mut Self, size: usize) {
        if size == self.size {
            return;
        };
        self.data = iter::repeat(T::default()).take(size).collect();
        self.size = size;
        self.head = 0;
        return;
    }

    /// Resizes the `RingBuffer`. This **copies** as much of  the previous
    /// buffer into the new buffer as possible. If you don't want to copy the
    /// previous values into the new buffer, use `resize()`.
    ///
    /// # Example
    ///
    /// ```
    /// use plext::utils::RingBuffer;
    /// let mut rb = RingBuffer::<f32>::new(2);
    /// rb.enqueue(1.0);
    /// rb.enqueue(2.0);
    /// rb.enqueue(3.0);
    /// // [2.0, 3.0]
    /// rb.resize(4);
    /// // [0.0, 0.0, 2.0, 3.0]
    /// rb.enqueue(4.0);
    /// // [0.0, 2.0, 3.0, 4.0]
    /// rb.resize(2);
    /// // [3.0, 4.0]
    /// ```
    pub fn resize_and_copy(self: &mut Self, size: usize) {
        if size == self.size {
            return;
        };

        let mut new_data: Vec<T> = iter::repeat(T::default()).take(size).collect();

        if size > self.size {
            for i in 0..self.size {
                new_data[i] = self.data[self.head];
                self.head += 1;
                self.head %= self.size;
            }
            self.head = self.size;
        } else {
            for i in 0..size {
                if self.head == 0 {
                    self.head = self.size;
                }
                self.head -= 1;
                println!("{:?}", self.data[self.head]);
                new_data[size - i - 1] = self.data[self.head];
            }
            self.head = 0;
        }

        self.data = new_data;
        self.size = size;
    }
}

impl<T: Clone + Copy + Default + std::fmt::Debug> IntoIterator for RingBuffer<T> {
    type Item = T;

    type IntoIter = RingBufferIntoIterator<T>;

    fn into_iter(self) -> Self::IntoIter {
        RingBufferIntoIterator {
            ring_buffer: self,
            index: 0,
        }
    }
}

pub struct RingBufferIntoIterator<T> {
    ring_buffer: RingBuffer<T>,
    index: usize,
}

impl<T: Clone + Copy + Default + std::fmt::Debug> Iterator for RingBufferIntoIterator<T> {
    type Item = T;
    fn next(&mut self) -> Option<T> {
        if self.index >= self.ring_buffer.size {
            return None;
        }
        self.index += 1;
        self.ring_buffer
            .data
            .get((self.index + self.ring_buffer.head - 1) % self.ring_buffer.size)
            .copied()
    }
}

impl<'a, T: Clone + Copy + Default + std::fmt::Debug> IntoIterator for &'a RingBuffer<T> {
    type Item = &'a T;

    type IntoIter = RingBufferRefIntoIterator<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        RingBufferRefIntoIterator {
            ring_buffer: self,
            index: 0,
        }
    }
}

pub struct RingBufferRefIntoIterator<'a, T> {
    ring_buffer: &'a RingBuffer<T>,
    index: usize,
}

impl<'a, T> Iterator for RingBufferRefIntoIterator<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<&'a T> {
        if self.index >= self.ring_buffer.size {
            return None;
        }
        self.index += 1;
        self.ring_buffer
            .data
            .get((self.index + self.ring_buffer.head - 1) % self.ring_buffer.size)
    }
}
