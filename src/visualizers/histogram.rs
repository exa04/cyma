use super::{FillFrom, FillModifiers, RangeModifiers};
use crate::utils::{HistogramBuffer, ValueScaling, VisualizerBuffer};

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
pub struct Histogram<L>
where
    L: Lens<Target = Arc<Mutex<HistogramBuffer>>>,
{
    buffer: L,
    range: (f32, f32),
}

enum HistogramEvents {
    UpdateRange((f32, f32)),
    // UpdateDecay(f32),
}

impl<L> Histogram<L>
where
    L: Lens<Target = Arc<Mutex<HistogramBuffer>>>,
{
    pub fn new(cx: &mut Context, buffer: L, range: impl Res<(f32, f32)> + Clone) -> Handle<Self> {
        Self {
            buffer,
            range: range.get_val(cx),
        }
        .build(cx, |_| {})
        .range(range)
    }
}

impl<L> View for Histogram<L>
where
    L: Lens<Target = Arc<Mutex<HistogramBuffer>>>,
{
    fn element(&self) -> Option<&'static str> {
        Some("histogram")
    }
    fn event(&mut self, _cx: &mut EventContext, event: &mut Event) {
        event.map(|e, _| match e {
            HistogramEvents::UpdateRange(v) => self.range = *v,
            // HistogramEvents::UpdateDecay(s) => self.decay = *s,
        });
    }
    fn draw(&self, cx: &mut DrawContext, canvas: &mut Canvas) {
        let bounds = cx.bounds();

        let line_width = cx.scale_factor();

        let x = bounds.x;
        let y = bounds.y;
        let w = bounds.w;
        let h = bounds.h;

        let mut stroke = vg::Path::new();
        let binding = self.buffer.get(cx);
        let bins = &(binding.lock().unwrap());
        let nr_bins = bins.len();

        let mut largest = 0.0;
        for i in 0..nr_bins {
            if bins[i] > largest {
                largest = bins[i];
            }
        }

        // start of the graph
        stroke.move_to(x + bins[nr_bins - 1] * w, y);

        // the actual histogram
        if largest > 0.0 {
            for i in 1..nr_bins {
                stroke.line_to(
                    x + (
                        // scale so the largest value becomes 1.
                        (bins[nr_bins - i] / largest) * w
                    ),
                    y + h * i as f32 / (nr_bins - 1) as f32,
                );
            }
        }
        // fill in with background color
        let mut fill = stroke.clone();
        fill.line_to(x, y + h);
        fill.line_to(x, y);
        fill.close();
        canvas.fill_path(&fill, &vg::Paint::color(cx.background_color().into()));

        canvas.stroke_path(
            &stroke,
            &vg::Paint::color(cx.font_color().into()).with_line_width(line_width),
        );
    }
}

impl<'a, L> FillModifiers for Handle<'a, Histogram<L>>
where
    L: Lens<Target = Arc<Mutex<HistogramBuffer>>>,
{
    // stubs
    fn fill_from_max(self) -> Self {
        self
    }
    fn fill_from_value(self, level: f32) -> Self {
        self
    }
}

impl<'a, L> RangeModifiers for Handle<'a, Histogram<L>>
where
    L: Lens<Target = Arc<Mutex<HistogramBuffer>>>,
{
    fn range(mut self, range: impl Res<(f32, f32)>) -> Self {
        let e = self.entity();

        range.set_or_bind(self.context(), e, move |cx, r| {
            (*cx).emit_to(e, HistogramEvents::UpdateRange(r.clone()));
        });

        self
    }

    fn scaling(mut self, scaling: impl Res<ValueScaling>) -> Self {
        // let e = self.entity();

        // scaling.set_or_bind(self.context(), e, move |cx, s| {
        // (*cx).emit_to(e, GraphEvents::UpdateScaling(s.clone()))
        // });

        self
    }
    // fn decay(mut self, decay: impl Res<f32>) -> Self {
    // let e = self.entity();

    // decay.set_or_bind(self.context(), e, move |cx, s| {
    // (*cx).emit_to(e, HistogramEvents::UpdateDecay(s.clone()))
    // });

    // self
    // }
}
