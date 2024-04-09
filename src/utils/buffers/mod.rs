pub mod peak_buffer;
pub mod ring_buffer;
pub mod waveform_buffer;

use std::ops::{Index, IndexMut};

pub use peak_buffer::PeakBuffer;
pub use ring_buffer::RingBuffer;
pub use waveform_buffer::WaveformBuffer;

pub trait VisualizerBuffer<T>: Index<usize> + IndexMut<usize> {
    /// Enqueues an element.
    ///
    /// Once enqueued, the value is situated at the tail of the buffer and the
    /// oldest element is removed from the head.
    fn enqueue(self: &mut Self, value: T);

    /// Enqueues an entire [`Buffer`](`nih_plug::buffer::Buffer`), mono-summing
    /// it if no channel is specified.
    fn enqueue_buffer(
        self: &mut Self,
        buffer: &mut nih_plug::buffer::Buffer,
        channel: Option<usize>,
    );

    /// Clears the entire buffer, filling it with default values (usually 0)
    fn clear(self: &mut Self);

    /// Grows the buffer to the provided size.
    ///
    /// The extra space is filled with the default values for your data type
    /// (usually 0). This operation keeps the order of the values intact.
    fn grow(self: &mut Self, size: usize);

    /// Shrinks the buffer to the provided size.
    ///
    /// The most recently enqueued elements are preserved. This operation keeps
    /// the order of the values intact.
    fn shrink(self: &mut Self, size: usize);

    /// Returns the length of the buffer.
    fn len(self: &Self) -> usize;

    /// Returns `true` if the buffer is empty.
    fn is_empty(self: &Self) -> bool {
        self.len() == 0
    }

    /// Resizes the buffer to the given size.
    ///
    /// Internally, this either calls [`shrink()`](`Buffer::shrink()`), or
    /// [`grow()`](`Buffer::grow()`), depending on the desired size.
    fn resize(self: &mut Self, size: usize) {
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
}
