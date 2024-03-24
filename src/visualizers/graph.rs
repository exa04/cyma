use len_trait::len::Len;
use std::{
    ops::Index,
    sync::{Arc, Mutex},
};

use nih_plug_vizia::vizia::{
    binding::{Lens, LensExt, Res},
    context::{Context, DrawContext},
    vg,
    view::{Canvas, Handle, View},
    views::normalized_map::amplitude_to_db,
};

/// A real-time graph displaying information that is stored inside a iterable
/// collection.
pub struct Graph<L, I>
where
    L: Lens<Target = Arc<Mutex<I>>>,
    I: Len + Index<usize, Output = f32>,
{
    buffer: L,
    display_range: (f32, f32),
    scale_by_db: bool,
}

impl<L, I> Graph<L, I>
where
    L: Lens<Target = Arc<Mutex<I>>>,
    I: Len + Index<usize, Output = f32> + 'static,
{
    pub fn new(
        cx: &mut Context,
        buffer: L,
        display_range: impl Res<(f32, f32)>,
        scale_by_db: impl Res<bool>,
    ) -> Handle<Self> {
        Self {
            buffer,
            display_range: display_range.get_val(cx),
            scale_by_db: scale_by_db.get_val(cx),
        }
        .build(cx, |_| {})
    }
}

impl<L, I> View for Graph<L, I>
where
    L: Lens<Target = Arc<Mutex<I>>>,
    I: Len + Index<usize, Output = f32> + 'static,
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

        // Peak graph
        let mut stroke = vg::Path::new();
        let binding = self.buffer.get(cx);
        let ring_buf = &(binding.lock().unwrap());

        let mut i = 1;

        if self.scale_by_db {
            let mut peak =
                (amplitude_to_db(ring_buf[0])).clamp(self.display_range.0, self.display_range.1);

            peak -= self.display_range.0;
            peak /= self.display_range.1 - self.display_range.0;

            stroke.move_to(x, y + h * (1. - peak));

            while i < ring_buf.len() {
                // Convert peak to decibels and clamp it in range
                peak = (amplitude_to_db(ring_buf[i]))
                    .clamp(self.display_range.0, self.display_range.1);

                // Normalize peak's range
                peak -= self.display_range.0;
                peak /= self.display_range.1 - self.display_range.0;

                // Draw peak as a new point
                stroke.line_to(
                    x + (w / ring_buf.len() as f32) * i as f32,
                    y + h * (1. - peak),
                );
                i += 1;
            }
        } else {
            let mut peak =
                (amplitude_to_db(ring_buf[0])).clamp(self.display_range.0, self.display_range.1);

            peak -= self.display_range.0;
            peak /= self.display_range.1 - self.display_range.0;

            stroke.move_to(x, y + h * (1. - peak));

            while i < ring_buf.len() {
                // Clamp peak in range
                let mut peak = ring_buf[i].clamp(self.display_range.0, self.display_range.1);

                // Normalize peak's range
                peak -= self.display_range.0;
                peak /= self.display_range.1 - self.display_range.0;

                // Draw peak as a new point
                stroke.line_to(
                    x + (w / ring_buf.len() as f32) * i as f32,
                    y + h * (1. - peak),
                );
                i += 1;
            }
        }

        let mut fill = stroke.clone();

        fill.line_to(x + w, y + h);
        fill.line_to(x, y + h);
        fill.close();

        canvas.fill_path(&fill, &vg::Paint::color(cx.background_color().into()));

        canvas.stroke_path(
            &stroke,
            &vg::Paint::color(cx.font_color().into()).with_line_width(line_width),
        );
    }
}
