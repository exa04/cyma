use std::sync::{Arc, Mutex};

use nih_plug_vizia::vizia::{prelude::*, vg};

use crate::utils::{ValueScaling, VisualizerBuffer, WaveformBuffer};

/// Waveform display for real-time input.
///
/// This visualizer is particularly useful when visualizing audio data at a
/// high sample rate, such as 44.1kHz, in a much smaller view. It does not naively
/// downsample the audio, which is why, even for very small sizes, it still
/// correctly displays the peak data.
///
/// # Example
///
/// ```
/// Oscilloscope::new(
///     cx,
///     Data::oscilloscope_buffer,
///     (-1.2, 1.2),
///     ValueScaling::Linear,
/// )
/// .color(Color::rgba(0, 0, 0, 0))
/// .background_color(Color::rgba(255, 255, 255, 120));
/// ```
///
pub struct Oscilloscope<B>
where
    B: Lens<Target = Arc<Mutex<WaveformBuffer>>>,
{
    buffer: B,
    display_range: (f32, f32),
    scaling: ValueScaling,
}

impl<B> Oscilloscope<B>
where
    B: Lens<Target = Arc<Mutex<WaveformBuffer>>>,
{
    /// Creates a new Oscilloscope.
    ///    
    /// Takes in a `buffer`, which should be used to store the peak values. You
    /// need to write to it inside your plugin code, thread-safely send it to
    /// the editor thread, and then pass it into this oscilloscope. Which is
    /// also why it is behind an `Arc<Mutex>`.
    pub fn new(
        cx: &mut Context,
        buffer: B,
        display_range: impl Res<(f32, f32)>,
        scaling: impl Res<ValueScaling>,
    ) -> Handle<Self> {
        Self {
            buffer,
            display_range: display_range.get_val(cx),
            scaling: scaling.get_val(cx),
        }
        .build(cx, |_| {})
    }
}

impl<B> View for Oscilloscope<B>
where
    B: Lens<Target = Arc<Mutex<WaveformBuffer>>>,
{
    fn element(&self) -> Option<&'static str> {
        Some("oscilloscope")
    }
    fn draw(&self, cx: &mut DrawContext, canvas: &mut Canvas) {
        let bounds = cx.bounds();

        let x = bounds.x;
        let y = bounds.y;
        let w = bounds.w;
        let h = bounds.h;

        let mut fill = vg::Path::new();

        let binding = self.buffer.get(cx);
        let ring_buf = &(binding.lock().unwrap());

        let width_delta = w / ring_buf.len() as f32;

        // Local minima (bottom part of waveform)
        let mut py = self.scaling.value_to_normalized(
            ring_buf[0].0,
            self.display_range.0,
            self.display_range.1,
        );
        fill.move_to(x, y + h * (1. - py) + 1.);
        for i in 1..ring_buf.len() {
            py = self.scaling.value_to_normalized(
                ring_buf[i].0,
                self.display_range.0,
                self.display_range.1,
            );

            fill.line_to(x + width_delta * i as f32, y + h * (1. - py) + 1.);
        }

        let bottom_stroke = fill.clone();
        let mut top_stroke = vg::Path::new();

        // Local maxima (top part of waveform)
        py = self.scaling.value_to_normalized(
            ring_buf[ring_buf.len() - 1].1,
            self.display_range.0,
            self.display_range.1,
        );
        fill.line_to(x + w, y + h * (1. - py) + 1.);
        top_stroke.move_to(x + w, y + h * (1. - py) + 1.);
        for i in 1..ring_buf.len() {
            py = self.scaling.value_to_normalized(
                ring_buf[ring_buf.len() - i].1,
                self.display_range.0,
                self.display_range.1,
            );

            fill.line_to(x + w - width_delta * i as f32, y + h * (1. - py) + 1.);
            top_stroke.line_to(x + w - width_delta * i as f32, y + h * (1. - py) + 1.);
        }

        fill.close();
        canvas.fill_path(
            &fill,
            &vg::Paint::color(cx.font_color().into()).with_line_width(0.),
        );
    }
}
