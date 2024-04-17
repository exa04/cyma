use std::sync::{Arc, Mutex};

use nih_plug_vizia::vizia::{prelude::*, vg};

use crate::utils::ValueScaling;
use crate::utils::VisualizerBuffer;

/// A Meter that displays the data inside a [`VisualizerBuffer`].
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
    display_range: (f32, f32),
    scaling: ValueScaling,
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
        display_range: impl Res<(f32, f32)>,
        scaling: impl Res<ValueScaling>,
        orientation: Orientation,
    ) -> Handle<Self> {
        Self {
            buffer,
            display_range: display_range.get_val(cx),
            scaling: scaling.get_val(cx),
            orientation,
        }
        .build(cx, |_| {})
    }
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
            self.display_range.0,
            self.display_range.1,
        );

        let mut path = vg::Path::new();
        match self.orientation {
            Orientation::Vertical => {
                path.move_to(x, y + h * (1. - level));
                path.line_to(x + w, y + h * (1. - level));

                let mut outline = path.clone();
                outline.close();
                canvas.fill_path(&outline, &vg::Paint::color(cx.font_color().into()));

                path.line_to(x + w, y + h);
                path.line_to(x, y + h);
                path.close();

                canvas.fill_path(&path, &vg::Paint::color(cx.background_color().into()));
            }
            Orientation::Horizontal => {
                path.move_to(x + w * level, y);
                path.line_to(x + w * level, y + h);

                let mut outline = path.clone();
                outline.close();
                canvas.fill_path(&outline, &vg::Paint::color(cx.font_color().into()));

                path.line_to(x, y + h);
                path.line_to(x, y);
                path.close();

                canvas.fill_path(&path, &vg::Paint::color(cx.background_color().into()));
            }
        };
    }
}
