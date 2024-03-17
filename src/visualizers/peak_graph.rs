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
        canvas.fill_path(
            &{
                let mut path = vg::Path::new();
                let binding = self.buffer.get(cx);
                let ring_buf = &(binding.lock().unwrap());

                path.move_to(x, y + h);

                let mut i = 0.;
                if self.scale_by_db {
                    for peak in ring_buf.into_iter() {
                        // Convert peak to decibels and clamp it in range
                        let mut peak = (amplitude_to_db(*peak))
                            .clamp(self.display_range.0, self.display_range.1);

                        // Normalize peak's range
                        peak -= self.display_range.0;
                        peak /= self.display_range.1 - self.display_range.0;

                        // Draw peak as a new point
                        path.line_to(x + (w / ring_buf.len() as f32) * i, y + h * (1. - peak));
                        i += 1.;
                    }
                } else {
                    for peak in ring_buf.into_iter() {
                        // Clamp peak in range
                        let mut peak = (*peak).clamp(self.display_range.0, self.display_range.1);

                        // Normalize peak's range
                        peak -= self.display_range.0;
                        peak /= self.display_range.1 - self.display_range.0;

                        // Draw peak as a new point
                        path.line_to(x + (w / ring_buf.len() as f32) * i, y + h * (1. - peak));
                        i += 1.;
                    }
                }
                path.line_to(x + w, y + h);
                path.close();
                path
            },
            &vg::Paint::color(cx.font_color().into()).with_line_width(line_width),
        );
    }
}
