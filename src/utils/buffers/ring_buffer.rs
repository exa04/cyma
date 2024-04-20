use std::fmt::Debug;
use std::ops::{Index, IndexMut};

/// A buffer that stores elements of type `T` in a First-In-First-Out manner.
///
/// The `RingBuffer` struct allows enqueueing new elements onto its tail. When
/// an element is enqueued, all older elements shift towards the head and the
/// oldest element is popped off the head of the buffer. Due to its fixed-size
/// nature, the ring buffer is very fast and doesn't dynamically reallocate
/// itself, or move any elements around when an element is added.
#[derive(Clone, PartialEq, Eq, Default, Hash, Debug)]
pub struct RingBuffer<T> {
    head: usize,
    size: usize,
    data: Vec<T>,
}

impl<T: Default + Copy> RingBuffer<T> {
    /// Constructs a new RingBuffer with the given size.
    pub fn new(size: usize) -> Self {
        Self {
            head: 0,
            size,
            data: vec![T::default(); size],
        }
    }

    /// Shrinks the RingBuffer to the given size.
    ///
    /// The most recently enqueued elements are preserved. This operation keeps
    /// the order of the values intact.
    pub fn shrink(self: &mut Self, size: usize) {
        let mut data = vec![];

        if size <= self.head {
            // Copy the last `size` elements before the head
            data.extend_from_slice(&self.data[self.head - size..self.head]);
        } else {
            // Copy the last `size` elements before the buffer wraps around
            data.extend_from_slice(&self.data[self.size - (size - self.head)..self.size]);
            // Copy everything before the head
            data.extend_from_slice(&self.data[0..self.head]);
        }

        self.head = 0;
        self.size = size;
        self.data = data;
    }

    /// Grows the RingBuffer.
    ///
    /// The extra space is filled with the default values for your data type
    /// (usually 0). This operation keeps the order of the values intact.
    pub fn grow(self: &mut Self, size: usize) {
        let mut data = vec![];

        // Copy everything after the head
        data.extend_from_slice(&self.data[self.head..self.size]);
        // Copy everything before the head
        data.extend_from_slice(&self.data[0..self.head]);

        for _ in self.size..size {
            data.push(T::default());
        }

        self.data = data;
        self.head = self.size;
        self.size = size;
    }

    /// Resizes the buffer to the given size.
    ///
    /// Internally, this either calls [`shrink()`](`Buffer::shrink()`), or
    /// [`grow()`](`Buffer::grow()`), depending on the desired size.
    pub fn resize(self: &mut Self, size: usize) {
        if size == self.len() {
            return;
        }
        if size < self.len() {
            self.shrink(size)
        }
        if size > self.len() {
            self.grow(size)
        }
    }

    /// Enqueues an element into the RingBuffer.
    ///
    /// Once enqueued, the value is situated at the tail of the buffer and the
    /// oldest element is removed from the head.
    pub fn enqueue(self: &mut Self, value: T) {
        self.data[self.head] = value;
        self.head = (self.head + 1) % self.size;
    }

    pub fn peek(self: &Self) -> T {
        self.data[(self.size + self.head - 1) % self.size]
    }

    /// Clears the entire buffer, filling it with default values (usually 0)
    pub fn clear(self: &mut Self) {
        self.data.iter_mut().for_each(|x| *x = T::default());
    }

    pub fn len(self: &Self) -> usize {
        self.size
    }
}

impl<T> Index<usize> for RingBuffer<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        if index >= self.size {
            panic!(
                "Invalid ring buffer access: Index {} is out of range for ring buffer of size {}",
                index, self.size
            );
        }
        &self.data[(self.head + index) % self.size]
    }
}
impl<T> IndexMut<usize> for RingBuffer<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        if index >= self.size {
            panic!(
                "Invalid ring buffer access: Index {} is out of range for ring buffer of size {}",
                index, self.size
            );
        }
        &mut self.data[(self.head + index) % self.size]
    }
}

#[cfg(test)]
mod tests {
    use super::RingBuffer;

    #[test]
    fn basics() {
        let mut rb = RingBuffer::<i32>::new(4);

        // Is the buffer filled with zeroes?
        assert_eq!(rb.data, vec![0; 4]);

        rb.enqueue(1);
        rb.enqueue(2);
        rb.enqueue(3);

        // Is the value at the tail (before the head) equal to 0?
        assert_eq!(rb.data[(rb.head + rb.size - 1) % rb.size], 3);

        // Is the value at the head equal to 0?
        assert_eq!(rb.data[rb.head], 0);

        rb.enqueue(4);
        rb.enqueue(5);
        rb.enqueue(6);

        // Have the earlier values been overwritten?
        assert!(!rb.data.contains(&1));
        assert!(!rb.data.contains(&2));

        // Do the last 4 values exist?
        assert!(rb.data.contains(&3));
        assert!(rb.data.contains(&4));
        assert!(rb.data.contains(&5));
        assert!(rb.data.contains(&6));
    }

    #[test]
    fn clear() {
        let mut rb = RingBuffer::<i32>::new(4);

        rb.enqueue(1);
        rb.enqueue(2);
        rb.enqueue(3);

        assert_ne!(rb.data, vec![0; 4]);

        rb.clear();

        assert_eq!(rb.data, vec![0; 4]);
    }

    #[test]
    fn resize() {
        let mut rb = RingBuffer::<i32>::new(4);
        rb.enqueue(1);
        rb.enqueue(2);
        rb.enqueue(3);
        rb.enqueue(4);
        rb.enqueue(5);
        rb.enqueue(6);

        // Growing RingBuffers
        {
            let mut rb_grown = rb.clone();
            rb_grown.grow(6);
            let mut rb_resized = rb.clone();
            rb_resized.resize(6);
            // Is the last inserted datum the same?
            assert_eq!(
                rb_grown.data[(rb_grown.head + rb_grown.size - 1) % rb_grown.size],
                rb_resized.data[(rb_resized.head + rb_resized.size - 1) % rb_resized.size]
            );
            // Is the buffer zero-padded?
            assert_eq!(rb_grown.data[rb_grown.head], 0);
        }

        // Shrinking RingBuffers
        {
            let mut rb_shrunk = rb.clone();
            rb_shrunk.shrink(3);
            let mut rb_resized = rb.clone();
            rb_resized.resize(3);
            // Is the last inserted datum the same?
            assert_eq!(
                rb_shrunk.data[(rb_shrunk.head + rb_shrunk.size - 1) % rb_shrunk.size],
                rb_resized.data[(rb_resized.head + rb_resized.size - 1) % rb_resized.size]
            );
        }
    }

    #[test]
    fn indexing() {
        let mut rb = RingBuffer::<i32>::new(4);

        rb.enqueue(1);
        rb.enqueue(2);
        rb.enqueue(3);

        // Is the last value still equal to 0?
        assert_eq!(rb[0], 0);

        // Were the first bunch of values inserted correctly?
        assert_eq!(rb[1], 1);
        assert_eq!(rb[2], 2);
        assert_eq!(rb[3], 3);

        rb[1] *= 2;
        rb[2] *= 3;
        rb[3] *= 4;

        // Were the values multiplied correctly?
        assert_eq!(rb[1], 2);
        assert_eq!(rb[2], 6);
        assert_eq!(rb[3], 12);

        rb.enqueue(4);
        rb.enqueue(5);

        // Have the newer values "pushed back" the older ones?
        assert_eq!(rb[0], 6);
        assert_eq!(rb[1], 12);
        assert_eq!(rb[2], 4);
        assert_eq!(rb[3], 5);

        // Can we set an element of the buffer?
        rb[2] = 10;
        assert_eq!(rb[2], 10);
    }

    #[test]
    #[should_panic]
    fn invalid_access() {
        let mut rb = RingBuffer::<i32>::new(4);

        rb.enqueue(1);
        rb.enqueue(2);
        rb.enqueue(3);

        rb[4];
    }

    #[test]
    fn peek() {
        let mut rb = RingBuffer::<i32>::new(4);

        rb.enqueue(1);
        assert_eq!(rb.peek(), 1);
        rb.enqueue(2);
        rb.enqueue(3);
        assert_eq!(rb.peek(), 3);
        rb.enqueue(4);
        rb.enqueue(5);
        rb.enqueue(6);
        rb.enqueue(7);
        assert_eq!(rb.peek(), 7);
    }
}
