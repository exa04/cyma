use super::RangeModifiers;
use crate::utils::{ValueScaling, VisualizerBuffer};

use nih_plug_vizia::vizia::{prelude::*, vg};
use std::sync::{Arc, Mutex};

/// Real-time graph displaying information that is stored inside a buffer
///
/// Use this view to construct peak graphs, loudness graphs, or any other graph that
/// displays the data inside a [`VisualizerBuffer`].
///
/// # Example
///
/// Here's how to set up a basic peak graph. For this example, you'll need a
/// [`PeakBuffer`](crate::utils::PeakBuffer) to store your peak information.
///
/// ```
/// Graph::new(cx, Data::peak_buffer, (-32.0, 8.0), ValueScaling::Decibels)
///     .color(Color::rgba(0, 0, 0, 160))
///     .background_color(Color::rgba(0, 0, 0, 60));
/// ```
///
/// The graph displays the range from -32.0dB to 8dB. It scales the values as
/// decibels, and a stroke and fill (background) color is provided.
pub struct Graph<L, I>
where
    L: Lens<Target = Arc<Mutex<I>>>,
    I: VisualizerBuffer<f32> + 'static,
{
    buffer: L,
    range: (f32, f32),
    scaling: ValueScaling,
    fill_from: FillFrom,
}

enum FillFrom {
    Top,
    Bottom,
    Value(f32),
}

enum GraphEvents {
    UpdateRange((f32, f32)),
}

impl<L, I> Graph<L, I>
where
    L: Lens<Target = Arc<Mutex<I>>>,
    I: VisualizerBuffer<f32, Output = f32> + 'static,
{
    pub fn new(
        cx: &mut Context,
        buffer: L,
        range: impl Res<(f32, f32)> + Clone,
        scaling: impl Res<ValueScaling> + Clone,
    ) -> Handle<Self> {
        let r = range.get_val(cx);
        Self {
            buffer,
            range: r,
            scaling: scaling.get_val(cx),
            fill_from: FillFrom::Bottom,
        }
        .build(cx, |_| {})
        .range(range)
    }
}

impl<L, I> View for Graph<L, I>
where
    L: Lens<Target = Arc<Mutex<I>>>,
    I: VisualizerBuffer<f32, Output = f32> + 'static,
{
    fn element(&self) -> Option<&'static str> {
        Some("graph")
    }
    fn draw(&self, cx: &mut DrawContext, canvas: &mut Canvas) {
        let bounds = cx.bounds();

        let x = bounds.x;
        let y = bounds.y;
        let w = bounds.w;
        let h = bounds.h;

        let line_width = cx.scale_factor();

        let mut stroke = vg::Path::new();
        let binding = self.buffer.get(cx);
        let ring_buf = &(binding.lock().unwrap());

        let mut peak = self
            .scaling
            .value_to_normalized(ring_buf[0], self.range.0, self.range.1);

        stroke.move_to(x, y + h * (1. - peak));

        for i in 1..ring_buf.len() {
            // Normalize peak value
            peak = self
                .scaling
                .value_to_normalized(ring_buf[i], self.range.0, self.range.1);

            // Draw peak as a new point
            stroke.line_to(
                x + (w / ring_buf.len() as f32) * i as f32,
                y + h * (1. - peak),
            );
        }

        let mut fill = stroke.clone();
        let fill_from_n = match self.fill_from {
            FillFrom::Top => 0.0,
            FillFrom::Bottom => 1.0,
            FillFrom::Value(val) => {
                1.0 - ValueScaling::Linear.value_to_normalized(val, self.range.0, self.range.1)
            }
        };

        fill.line_to(x + w, y + h * fill_from_n);
        fill.line_to(x, y + h * fill_from_n);
        fill.close();

        canvas.fill_path(&fill, &vg::Paint::color(cx.background_color().into()));

        canvas.stroke_path(
            &stroke,
            &vg::Paint::color(cx.font_color().into()).with_line_width(line_width),
        );
    }
    fn event(
        &mut self,
        _cx: &mut nih_plug_vizia::vizia::context::EventContext,
        event: &mut nih_plug_vizia::vizia::events::Event,
    ) {
        event.map(|e, _| match e {
            GraphEvents::UpdateRange(v) => self.range = *v,
        });
    }
}

pub trait GraphModifiers {
    /// Allows for the graph to be filled from the top instead of the bottom.
    ///
    /// This is useful for certain graphs like gain reduction meters.
    ///
    /// # Example
    ///
    /// Here's a gain reduction graph, which you could overlay on top of a peak graph.
    ///
    /// Here, `gain_mult` could be a [`MinimaBuffer`](crate::utils::MinimaBuffer).
    ///
    /// ```
    /// Graph::new(cx, Data::gain_mult, (-32.0, 8.0), ValueScaling::Decibels)
    ///     .fill_from_top()
    ///     .color(Color::rgba(255, 0, 0, 160))
    ///     .background_color(Color::rgba(255, 0, 0, 60));
    /// ```
    fn fill_from_top(self) -> Self;

    /// Allows for the graph to be filled from any desired level.
    ///
    /// This is useful for certain graphs like gain reduction meters.
    ///
    /// # Example
    ///
    /// Here's a gain reduction graph, which you could overlay on top of a peak graph.
    ///
    /// Here, `gain_mult` could be a [`MinimaBuffer`](crate::utils::MinimaBuffer).
    ///
    /// ```
    /// Graph::new(cx, Data::gain_mult, (-32.0, 6.0), ValueScaling::Decibels)
    ///     .fill_from(0.0) // Fills the graph from 0.0dB downwards
    ///     .color(Color::rgba(255, 0, 0, 160))
    ///     .background_color(Color::rgba(255, 0, 0, 60));
    /// ```
    fn fill_from_value(self, level: f32) -> Self;
}

impl<'a, L, I> GraphModifiers for Handle<'a, Graph<L, I>>
where
    L: Lens<Target = Arc<Mutex<I>>>,
    I: VisualizerBuffer<f32, Output = f32> + 'static,
{
    fn fill_from_top(self) -> Self {
        self.modify(|graph| {
            graph.fill_from = FillFrom::Top;
        })
    }
    fn fill_from_value(self, level: f32) -> Self {
        self.modify(|graph| {
            graph.fill_from = FillFrom::Value(level);
        })
    }
}

impl<'a, L, I> RangeModifiers for Handle<'a, Graph<L, I>>
where
    L: Lens<Target = Arc<Mutex<I>>>,
    I: VisualizerBuffer<f32, Output = f32> + 'static,
{
    fn range(mut self, range: impl Res<(f32, f32)>) -> Self {
        let e = self.entity();

        range.set_or_bind(self.context(), e, move |cx, r| {
            (*cx).emit_to(e, GraphEvents::UpdateRange(r.clone()));
        });

        self
    }
}
