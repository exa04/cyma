use std::sync::{Arc, Mutex};

use nih_plug_vizia::vizia::{prelude::*, vg};

use super::{FillFrom, FillModifiers, RangeModifiers};
use crate::utils::ValueScaling;
use crate::utils::VisualizerBuffer;

/// Meter that displays the data inside a [`VisualizerBuffer`].
///
/// Useful for peak meters, loudness meters, etc.
///
/// # Example
///
/// ```
/// Meter::new(
///     cx,
///     Data::peak_buffer,
///     (-32.0, 8.0),
///     ValueScaling::Decibels,
///     Orientation::Vertical,
/// )
/// .width(Pixels(24.0))
/// .height(Pixels(128.0))
/// .background_color(Color::rgb(100, 100, 100));
/// ```
pub struct Meter<L, I>
where
    L: Lens<Target = Arc<Mutex<I>>>,
    I: VisualizerBuffer<f32, Output = f32> + 'static,
{
    buffer: L,
    range: (f32, f32),
    scaling: ValueScaling,
    fill_from: FillFrom,
    orientation: Orientation,
}

impl<L, I> Meter<L, I>
where
    L: Lens<Target = Arc<Mutex<I>>>,
    I: VisualizerBuffer<f32, Output = f32> + 'static,
{
    pub fn new(
        cx: &mut Context,
        buffer: L,
        range: impl Res<(f32, f32)>,
        scaling: impl Res<ValueScaling>,
        orientation: Orientation,
    ) -> Handle<Self> {
        Self {
            buffer,
            range: range.get_val(cx),
            scaling: scaling.get_val(cx),
            fill_from: FillFrom::Bottom,
            orientation,
        }
        .build(cx, |_| {})
        .range(range)
        .scaling(scaling)
    }
}

enum MeterEvents {
    UpdateRange((f32, f32)),
    UpdateScaling(ValueScaling),
}

impl<L, I> View for Meter<L, I>
where
    L: Lens<Target = Arc<Mutex<I>>>,
    I: VisualizerBuffer<f32, Output = f32> + 'static,
{
    fn element(&self) -> Option<&'static str> {
        Some("meter")
    }
    fn draw(&self, cx: &mut DrawContext, canvas: &mut Canvas) {
        let bounds = cx.bounds();

        let x = bounds.x;
        let y = bounds.y;
        let w = bounds.w;
        let h = bounds.h;

        let binding = self.buffer.get(cx);
        let ring_buf = &(binding.lock().unwrap());

        let level = self.scaling.value_to_normalized(
            ring_buf[ring_buf.len() - 1],
            self.range.0,
            self.range.1,
        );

        let mut path = vg::Path::new();
        match self.orientation {
            Orientation::Vertical => {
                path.move_to(x, y + h * (1. - level));
                path.line_to(x + w, y + h * (1. - level));

                let mut outline = path.clone();
                outline.close();
                canvas.fill_path(&outline, &vg::Paint::color(cx.font_color().into()));

                let fill_from_n = match self.fill_from {
                    FillFrom::Top => 0.0,
                    FillFrom::Bottom => 1.0,
                    FillFrom::Value(val) => {
                        1.0 - ValueScaling::Linear.value_to_normalized(
                            val,
                            self.range.0,
                            self.range.1,
                        )
                    }
                };

                path.line_to(x + w, y + h * fill_from_n);
                path.line_to(x, y + h * fill_from_n);
                path.close();

                canvas.fill_path(&path, &vg::Paint::color(cx.background_color().into()));
            }
            Orientation::Horizontal => {
                path.move_to(x + w * level, y);
                path.line_to(x + w * level, y + h);

                let mut outline = path.clone();
                outline.close();
                canvas.fill_path(&outline, &vg::Paint::color(cx.font_color().into()));

                let fill_from_n = match self.fill_from {
                    FillFrom::Top => 1.0,
                    FillFrom::Bottom => 0.0,
                    FillFrom::Value(val) => {
                        ValueScaling::Linear.value_to_normalized(val, self.range.0, self.range.1)
                    }
                };

                path.line_to(x + w * fill_from_n, y + h);
                path.line_to(x + w * fill_from_n, y);
                path.close();

                canvas.fill_path(&path, &vg::Paint::color(cx.background_color().into()));
            }
        };
    }
    fn event(&mut self, _cx: &mut EventContext, event: &mut Event) {
        event.map(|e, _| match e {
            MeterEvents::UpdateRange(v) => self.range = *v,
            MeterEvents::UpdateScaling(v) => self.scaling = *v,
        });
    }
}

impl<'a, L, I> FillModifiers for Handle<'a, Meter<L, I>>
where
    L: Lens<Target = Arc<Mutex<I>>>,
    I: VisualizerBuffer<f32, Output = f32> + 'static,
{
    /// Allows for the meter to be filled from the maximum instead of the minimum value.
    ///
    /// This is useful for certain meters like gain reduction meters.
    ///
    /// # Example
    ///
    /// Here's a gain reduction meter, which you could overlay on top of a peak meter.
    ///
    /// Here, `gain_mult` could be a [`MinimaBuffer`](crate::utils::MinimaBuffer).
    ///
    /// ```
    /// Meter::new(cx, Data::gain_mult, (-32.0, 8.0), ValueScaling::Decibels, Orientation::Vertical)
    ///     .fill_from_max()
    ///     .color(Color::rgba(255, 0, 0, 160))
    ///     .background_color(Color::rgba(255, 0, 0, 60));
    /// ```
    fn fill_from_max(self) -> Self {
        self.modify(|meter| {
            meter.fill_from = FillFrom::Top;
        })
    }
    /// Allows for the meter to be filled from any desired level.
    ///
    /// This is useful for certain meters like gain reduction meters.
    ///
    /// # Example
    ///
    /// Here's a gain reduction meter, which you could overlay on top of a peak meter.
    ///
    /// Here, `gain_mult` could be a [`MinimaBuffer`](crate::utils::MinimaBuffer).
    ///
    /// ```
    /// Meter::new(cx, Data::gain_mult, (-32.0, 6.0), ValueScaling::Decibels, Orientation::Vertical)
    ///     .fill_from(0.0) // Fills the meter from 0.0dB downwards
    ///     .color(Color::rgba(255, 0, 0, 160))
    ///     .background_color(Color::rgba(255, 0, 0, 60));
    /// ```
    fn fill_from_value(self, level: f32) -> Self {
        self.modify(|meter| {
            meter.fill_from = FillFrom::Value(level);
        })
    }
}

impl<'a, L, I> RangeModifiers for Handle<'a, Meter<L, I>>
where
    L: Lens<Target = Arc<Mutex<I>>>,
    I: VisualizerBuffer<f32, Output = f32> + 'static,
{
    fn range(mut self, range: impl Res<(f32, f32)>) -> Self {
        let e = self.entity();

        range.set_or_bind(self.context(), e, move |cx, r| {
            (*cx).emit_to(e, MeterEvents::UpdateRange(r));
        });

        self
    }
    fn scaling(mut self, scaling: impl Res<ValueScaling>) -> Self {
        let e = self.entity();

        scaling.set_or_bind(self.context(), e, move |cx, s| {
            (*cx).emit_to(e, MeterEvents::UpdateScaling(s));
        });

        self
    }
}
