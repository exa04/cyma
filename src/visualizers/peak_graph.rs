use std::sync::{Arc, Mutex};

use crate::utils::PeakBuffer;
use nih_plug_vizia::vizia::{
    binding::{Lens, LensExt, Res},
    context::{Context, DrawContext},
    vg,
    view::{Canvas, Handle, View},
    views::normalized_map::amplitude_to_db,
};

/// A graph showing real-time peak information.
pub struct PeakGraph<B>
where
    B: Lens<Target = Arc<Mutex<PeakBuffer<f32>>>>,
{
    buffer: B,
    display_range: (f32, f32),
    scale_by_db: bool,
}

impl<B> PeakGraph<B>
where
    B: Lens<Target = Arc<Mutex<PeakBuffer<f32>>>>,
{
    pub fn new(
        cx: &mut Context,
        buffer: B,
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

impl<B> View for PeakGraph<B>
where
    B: Lens<Target = Arc<Mutex<PeakBuffer<f32>>>>,
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
        let mut rb_iter = ring_buf.into_iter();

        let mut i = 0.;
        if self.scale_by_db {
            let mut peak = (amplitude_to_db(*(rb_iter.next().unwrap())))
                .clamp(self.display_range.0, self.display_range.1);

            peak -= self.display_range.0;
            peak /= self.display_range.1 - self.display_range.0;

            stroke.move_to(x, y + h * (1. - peak));

            for p in rb_iter {
                // Convert peak to decibels and clamp it in range
                peak = (amplitude_to_db(*p)).clamp(self.display_range.0, self.display_range.1);

                // Normalize peak's range
                peak -= self.display_range.0;
                peak /= self.display_range.1 - self.display_range.0;

                // Draw peak as a new point
                stroke.line_to(x + (w / ring_buf.len() as f32) * i, y + h * (1. - peak));
                i += 1.;
            }
        } else {
            let mut peak =
                (*(rb_iter.next().unwrap())).clamp(self.display_range.0, self.display_range.1);

            peak -= self.display_range.0;
            peak /= self.display_range.1 - self.display_range.0;

            stroke.move_to(x, y + h * (1. - peak));

            for peak in rb_iter {
                // Clamp peak in range
                let mut peak = (*peak).clamp(self.display_range.0, self.display_range.1);

                // Normalize peak's range
                peak -= self.display_range.0;
                peak /= self.display_range.1 - self.display_range.0;

                // Draw peak as a new point
                stroke.line_to(x + (w / ring_buf.len() as f32) * i, y + h * (1. - peak));
                i += 1.;
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
