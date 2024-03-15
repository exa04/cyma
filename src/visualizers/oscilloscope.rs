use std::sync::{Arc, Mutex};

use nih_plug_vizia::vizia::{prelude::*, vg};

use crate::utils::MaximaBuffer;

/// Displays a waveform, retaining peak details for all frequencies within the
/// sample rate, regardless of buffer size.
///
/// This visualizer is particularly useful when visualizing audio data at a
/// high sample rate, such as 44.1kHz, in a much smaller view. It does not
/// downsample the audio, which is why, even for very small sizes, it still
/// correctly displays the peak data.
///
/// # How to use
///
/// To use this Visualizer, you need a [`MaximaBuffer`](`crate::utils::MaximaBuffer`)
/// that you write to inside your plugin code, and then send to the editor
/// thread - wrap it in an `Arc<Mutex>` to send it.
///
pub struct Oscilloscope<B>
where
    B: Lens<Target = Arc<Mutex<MaximaBuffer<f32>>>>,
{
    buffer: B,
}

impl<B> Oscilloscope<B>
where
    B: Lens<Target = Arc<Mutex<MaximaBuffer<f32>>>>,
{
    /// Creates a new Oscilloscope.
    ///    
    /// Takes in a `buffer`, which should be used to store the peak values. You
    /// need to write to it inside your plugin code, thread-safely send it to
    /// the editor thread, and then pass it into this oscilloscope. Which is
    /// also why it is behind an `Arc<Mutex>`.
    pub fn new(cx: &mut Context, buffer: B) -> Handle<Self> {
        Self { buffer }.build(cx, |_| {})
    }
}

impl<B> View for Oscilloscope<B>
where
    B: Lens<Target = Arc<Mutex<MaximaBuffer<f32>>>>,
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

        let line_width = cx.scale_factor();

        // Waveform
        canvas.fill_path(
            &{
                let mut path = vg::Path::new();
                let binding = self.buffer.get(cx);
                let ring_buf = &(binding.lock().unwrap());

                path.move_to(x, y + h / 2.);

                let mut i = 0.;
                for v in ring_buf.into_iter() {
                    path.line_to(
                        x + (w / ring_buf.len() as f32) * i,
                        y + (h / 2.) * (1. - v.0) + 1.,
                    );
                    i += 1.;
                }
                for v in ring_buf.into_iter().rev() {
                    i -= 1.;
                    path.line_to(
                        x + (w / ring_buf.len() as f32) * i,
                        y + (h / 2.) * (1. - v.1) + 1.,
                    );
                }
                path.close();
                path
            },
            &vg::Paint::color(cx.font_color().into()).with_line_width(line_width),
        );
    }
}

/// Displays a grid with a given number of subdivisions.
///
/// This is intended for use alongside the [`Oscilloscope`]. Put this grid
/// behind an oscilloscope using a [`ZStack`] to display a grid behind it.
pub struct OscilloscopeGrid {
    subdivisions: f32,
}

impl OscilloscopeGrid {
    /// Creates a new `OscilloscopeGrid` with the provided amount of
    /// subdivisions.
    ///
    /// If you have 4.2 seconds of audio, `subdivisions` could be `4.2`. If
    /// you have 8 bars of audio, it could be `8`.
    pub fn new<L: Res<f32> + Clone>(cx: &mut Context, length: L) -> Handle<Self> {
        Self {
            subdivisions: length.get_val(cx),
        }
        .build(cx, |_| {})
    }
}

impl View for OscilloscopeGrid {
    fn element(&self) -> Option<&'static str> {
        Some("22-oscilloscope-grid")
    }
    fn draw(&self, cx: &mut DrawContext, canvas: &mut Canvas) {
        let bounds = cx.bounds();

        let x = bounds.x;
        let y = bounds.y;
        let w = bounds.w;
        let h = bounds.h;

        let line_width = cx.scale_factor();

        canvas.stroke_path(
            &{
                let mut path = vg::Path::new();

                for level in (1..3).map(|x| x as f32 / 2.) {
                    let ry = level * ((h) / 2.);
                    path.move_to(x, y + h / 2. - ry);
                    path.line_to(x + w, y + h / 2. - ry);
                    path.close();
                    path.move_to(x, y + h / 2. + ry);
                    path.line_to(x + w, y + h / 2. + ry);
                    path.close();
                }
                path.close();

                path
            },
            &vg::Paint::color(cx.font_color().into()).with_line_width(line_width),
        );

        canvas.stroke_path(
            &{
                let mut path = vg::Path::new();

                let length = self.subdivisions;
                let t_delta = w / length;

                for step in (0..length.ceil() as u32).map(|x| x as f32 * t_delta) {
                    path.move_to(x + w - step, y);
                    path.line_to(x + w - step, y + h);
                    path.close();
                }

                path
            },
            &vg::Paint::color(cx.font_color().into()).with_line_width(line_width),
        );
    }
}
