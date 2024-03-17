use std::sync::{Arc, Mutex};

use nih_plug_vizia::vizia::{prelude::*, vg};

use crate::utils::WaveformBuffer;

// TODO: Allow setting a range, analogous to PeakGraph

/// A waveform display for real-time input.
///
/// This visualizer is particularly useful when visualizing audio data at a
/// high sample rate, such as 44.1kHz, in a much smaller view. It does not
/// downsample the audio, which is why, even for very small sizes, it still
/// correctly displays the peak data.
///
/// # How to use
///
/// To use this Visualizer, you need a [`WaveformBuffer`](`crate::utils::WaveformBuffer`)
/// that you write to inside your plugin code, and then send to the editor
/// thread - wrap it in an `Arc<Mutex>` to send it.
///
pub struct Oscilloscope<B>
where
    B: Lens<Target = Arc<Mutex<WaveformBuffer<f32>>>>,
{
    buffer: B,
    max: f32,
}

impl<B> Oscilloscope<B>
where
    B: Lens<Target = Arc<Mutex<WaveformBuffer<f32>>>>,
{
    /// Creates a new Oscilloscope.
    ///    
    /// Takes in a `buffer`, which should be used to store the peak values. You
    /// need to write to it inside your plugin code, thread-safely send it to
    /// the editor thread, and then pass it into this oscilloscope. Which is
    /// also why it is behind an `Arc<Mutex>`.
    pub fn new(cx: &mut Context, buffer: B, max: impl Res<f32>) -> Handle<Self> {
        Self {
            buffer,
            max: max.get_val(cx),
        }
        .build(cx, |_| {})
    }
}

impl<B> View for Oscilloscope<B>
where
    B: Lens<Target = Arc<Mutex<WaveformBuffer<f32>>>>,
{
    fn element(&self) -> Option<&'static str> {
        Some("22-visualizer")
    }
    fn draw(&self, cx: &mut DrawContext, canvas: &mut Canvas) {
        let bounds = cx.bounds();

        let x = bounds.x;
        let y = bounds.y;
        let w = bounds.w;
        let h = bounds.h;

        // Waveform
        canvas.fill_path(
            &{
                let mut path = vg::Path::new();
                let binding = self.buffer.get(cx);
                let ring_buf = &(binding.lock().unwrap());

                path.move_to(x, y + h / 2.);

                let mut i = 0.;
                for v in ring_buf.into_iter() {
                    // Clamp value to be in range
                    let mut py = v.0.clamp(-self.max, self.max);
                    // Normalize value
                    py /= self.max;

                    path.line_to(
                        x + (w / ring_buf.len() as f32) * i,
                        y + (h / 2.) * (1. - py) + 1.,
                    );
                    i += 1.;
                }
                for v in ring_buf.into_iter().rev() {
                    // Clamp value to be in range
                    let mut py = v.1.clamp(-self.max, self.max);
                    // Normalize value
                    py /= self.max;

                    path.line_to(
                        x + (w / ring_buf.len() as f32) * i,
                        y + (h / 2.) * (1. - py) + 1.,
                    );
                    i -= 1.;
                }
                path.close();
                path
            },
            &vg::Paint::color(cx.font_color().into()),
        );
    }
}
