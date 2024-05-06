use std::sync::{Arc, Mutex};

use nih_plug_vizia::vizia::{prelude::*, vg};

use super::RangeModifiers;
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
    range: (f32, f32),
    scaling: ValueScaling,
}

enum OscilloscopeEvents {
    UpdateRange((f32, f32)),
    UpdateScaling(ValueScaling),
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
        range: impl Res<(f32, f32)>,
        scaling: impl Res<ValueScaling>,
    ) -> Handle<Self> {
        Self {
            buffer,
            range: range.get_val(cx),
            scaling: scaling.get_val(cx),
        }
        .build(cx, |_| {})
        .range(range)
        .scaling(scaling)
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
        let mut py = self
            .scaling
            .value_to_normalized(ring_buf[0].0, self.range.0, self.range.1);
        fill.move_to(x, y + h * (1. - py) + 1.);
        for i in 1..ring_buf.len() {
            py = self
                .scaling
                .value_to_normalized(ring_buf[i].0, self.range.0, self.range.1);

            fill.line_to(x + width_delta * i as f32, y + h * (1. - py) + 1.);
        }

        let bottom_stroke = fill.clone();
        let mut top_stroke = vg::Path::new();

        // Local maxima (top part of waveform)
        py = self.scaling.value_to_normalized(
            ring_buf[ring_buf.len() - 1].1,
            self.range.0,
            self.range.1,
        );
        fill.line_to(x + w, y + h * (1. - py) + 1.);
        top_stroke.move_to(x + w, y + h * (1. - py) + 1.);
        for i in 1..ring_buf.len() {
            py = self.scaling.value_to_normalized(
                ring_buf[ring_buf.len() - i].1,
                self.range.0,
                self.range.1,
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
    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        event.map(|e, _| match e {
            OscilloscopeEvents::UpdateRange(v) => self.range = *v,
            OscilloscopeEvents::UpdateScaling(v) => self.scaling = *v,
        });
    }
}

impl<'a, B> RangeModifiers for Handle<'a, Oscilloscope<B>>
where
    B: Lens<Target = Arc<Mutex<WaveformBuffer>>>,
{
    fn range(mut self, range: impl Res<(f32, f32)>) -> Self {
        let e = self.entity();

        range.set_or_bind(self.context(), e, move |cx, r| {
            (*cx).emit_to(e, OscilloscopeEvents::UpdateRange(r));
        });

        self
    }
    fn scaling(mut self, scaling: impl Res<ValueScaling>) -> Self {
        let e = self.entity();

        scaling.set_or_bind(self.context(), e, move |cx, s| {
            (*cx).emit_to(e, OscilloscopeEvents::UpdateScaling(s));
        });

        self
    }
}
