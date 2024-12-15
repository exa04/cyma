pub mod peak_buffer;

use std::ops::{Index, IndexMut};

pub use crate::utils::ring_buffer::RingBuffer;
use crate::utils::MonoChannelConsumer;
pub use peak_buffer::PeakBuffer;

/// Common trait for buffers used by visualizers.
pub trait VisualizerBuffer<T: Default + Copy> {
    fn inner_buffer(&mut self) -> &mut RingBuffer<T>;

    fn consumer(&mut self) -> &mut MonoChannelConsumer;

    fn enqueue(&mut self, value: f32);

    /// Clears the entire buffer, filling it with default values (usually 0)
    fn clear(&mut self) {
        self.inner_buffer().clear();
    }

    /// Grows the buffer to the provided size.
    ///
    /// The extra space is filled with the default values for your data type
    /// (usually 0). This operation keeps the order of the values intact.
    #[inline]
    fn grow(&mut self, size: usize) {
        self.inner_buffer().grow(size);
    }

    /// Shrinks the buffer to the provided size.
    ///
    /// The most recently enqueued elements are preserved. This operation keeps
    /// the order of the values intact.
    #[inline]
    fn shrink(&mut self, size: usize) {
        self.inner_buffer().shrink(size);
    }

    /// Returns the length of the buffer.
    #[inline]
    fn len(&mut self) -> usize {
        self.inner_buffer().len()
    }

    /// Returns `true` if the buffer is empty.
    #[inline]
    fn is_empty(&mut self) -> bool {
        self.len() == 0
    }

    /// Resizes the buffer to the given size.
    ///
    /// Internally, this either calls [`shrink()`](`Buffer::shrink()`), or
    /// [`grow()`](`Buffer::grow()`), depending on the desired size.
    #[inline]
    fn resize(&mut self, size: usize) {
        self.inner_buffer().resize(size);
    }
}

/// Common trait for buffers used by the graph.
pub trait GraphBuffer: VisualizerBuffer<f32> {
    fn sample_rate(&self) -> f32;
    fn set_sample_rate(&mut self, sample_rate: f32);
    /// Updates the buffer.
    fn enqueue_latest(&mut self) {
        let sample_rate = self.consumer().get_sample_rate();

        if sample_rate != self.sample_rate() {
            self.set_sample_rate(sample_rate);
        }

        while let Some(sample) = self.consumer().receive() {
            self.enqueue(sample);
        }
    }
}
