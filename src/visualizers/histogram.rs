use super::{FillFrom, FillModifiers, RangeModifiers};
use crate::utils::HistogramBuffer;

use nih_plug_vizia::vizia::{prelude::*, vg};
use std::sync::{Arc, Mutex};

/// Real-time histogram displaying information that is stored inside a [`HistogramBuffer`]
///
/// # Example
///
/// Here's how to set up a histogram. For this example, you'll need a
/// [`HistogramBuffer`](crate::utils::HistogramBuffer) to store your histogram information.
///
/// ```
/// Histogram::new(cx, Data::histogram_buffer, (-32.0, 8.0), 0.1)
///     .color(Color::rgba(0, 0, 0, 160))
///     .background_color(Color::rgba(0, 0, 0, 60));
/// ```
///
/// The histogram displays the range from -32.0dB to 8dB.
/// it decays as 0.1 TODO, and a stroke and fill (background) color is provided.
pub struct Histogram<L, I>
where
    L: Lens<Target = Arc<Mutex<I>>>,
    I: HistogramBuffer + 'static,
{
    buffer: L,
    range: (f32, f32),
    decay: f32,
}

enum HistogramEvents {
    UpdateRange((f32, f32)),
    UpdateDecay(f32),
}

impl<L, I> Histogram<L, I>
where
    L: Lens<Target = Arc<Mutex<I>>>,
    I: HistogramBuffer + 'static,
{
    pub fn new(
        cx: &mut Context,
        buffer: L,
        range: impl Res<(f32, f32)> + Clone,
        decay: impl Res<f32> + Clone,
    ) -> Handle<Self> {
        Self {
            buffer,
            range: range.get_val(cx),
            decay: decay.get_val(cx),
        }
        .build(cx, |_| {})
        .range(range)
        .decay(decay)
    }
}

impl<L, I> View for Histogram<L, I>
where
    L: Lens<Target = Arc<Mutex<I>>>,
    I: HistogramBuffer + 'static,
{
    fn element(&self) -> Option<&'static str> {
        Some("histogram")
    }
    fn event(&mut self, _cx: &mut EventContext, event: &mut Event) {
        event.map(|e, _| match e {
            HistogramEvents::UpdateRange(v) => self.range = *v,
            HistogramEvents::UpdateDecay(s) => self.decay = *s,
        });
    }
    fn draw(&self, cx: &mut DrawContext, canvas: &mut Canvas) {
        let bounds = cx.bounds();

        let line_width = cx.scale_factor();

        let x = bounds.x + line_width / 2.0;
        let y = bounds.y + line_width / 2.0;
        let w = bounds.w - line_width;
        let h = bounds.h - line_width;

        let mut stroke = vg::Path::new();
        let binding = self.buffer.get(cx);
        let bins = &(binding.lock().unwrap());
        let nr_bins = self.buffer.size;

        // start of the graph
        stroke.move_to(x + bins[0] * w, y);
        for i in 1..nr_bins {
            stroke.line_to(x + bins[i] * w, y + h * i as f32 / (nr_bins - 1) as f32);
        }

        // let mut fill = stroke.clone();

        // fill.line_to(x + w, y + h * fill_from_n);
        // fill.line_to(x, y + h * fill_from_n);
        // fill.close();

        // canvas.fill_path(&fill, &vg::Paint::color(cx.background_color().into()));

        canvas.stroke_path(
            &stroke,
            &vg::Paint::color(cx.font_color().into()).with_line_width(line_width),
        );
    }
}

impl<'a, L, I> FillModifiers for Handle<'a, Histogram<L, I>>
where
    L: Lens<Target = Arc<Mutex<I>>>,
    I: HistogramBuffer + 'static,
{
}

// impl<'a, L, I> RangeModifiers for Handle<'a, Histogram<L, I>>
// where
// L: Lens<Target = Arc<Mutex<I>>>,
// I: HistogramBuffer + 'static,
// {
// fn range(mut self, range: impl Res<(f32, f32)>) -> Self {
// let e = self.entity();

// range.set_or_bind(self.context(), e, move |cx, r| {
// (*cx).emit_to(e, HistogramEvents::UpdateRange(r.clone()));
// });

// self
// }
// fn decay(mut self, decay: impl Res<f32>) -> Self {
// let e = self.entity();

// decay.set_or_bind(self.context(), e, move |cx, s| {
// (*cx).emit_to(e, HistogramEvents::UpdateDecay(s.clone()))
// });

// self
// }
// }
