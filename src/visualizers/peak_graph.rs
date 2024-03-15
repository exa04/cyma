use std::sync::{Arc, Mutex};

use crate::utils::PeakBuffer;
use nih_plug_vizia::vizia::{
    binding::{Lens, LensExt, Res},
    context::{Context, DrawContext},
    vg,
    view::{Canvas, Handle, View},
    views::normalized_map::{amplitude_to_db, db_to_amplitude},
};

pub struct PeakGraph<B, R, D>
where
    B: Lens<Target = Arc<Mutex<PeakBuffer<f32>>>>,
    R: Lens<Target = (f32, f32)>,
    D: Lens<Target = bool>,
{
    buffer: B,
    display_range: R,
    scale_by_db: D,
}

impl<B, R, D> PeakGraph<B, R, D>
where
    B: Lens<Target = Arc<Mutex<PeakBuffer<f32>>>>,
    R: Lens<Target = (f32, f32)>,
    D: Lens<Target = bool>,
{
    pub fn new(cx: &mut Context, buffer: B, display_range: R, scale_by_db: D) -> Handle<Self> {
        Self {
            buffer,
            display_range,
            scale_by_db,
        }
        .build(cx, |_| {})
    }
}

impl<B, R, D> View for PeakGraph<B, R, D>
where
    B: Lens<Target = Arc<Mutex<PeakBuffer<f32>>>>,
    R: Lens<Target = (f32, f32)>,
    D: Lens<Target = bool>,
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
                let range = self.display_range.get(cx);

                path.move_to(x, y + h);

                let mut i = 0.;
                if self.scale_by_db.get(cx) {
                    for v in ring_buf.into_iter() {
                        path.line_to(
                            x + (w / ring_buf.len() as f32) * i,
                            y + h
                                * ({
                                    // Convert to decibels, clamp and then transform to be between 1. and 0.
                                    1. - ((amplitude_to_db(*v)).clamp(range.0, range.1) - range.0)
                                        / (range.1 - range.0)
                                }),
                        );
                        i += 1.;
                    }
                } else {
                    for v in ring_buf.into_iter() {
                        path.line_to(
                            x + (w / ring_buf.len() as f32) * i,
                            y + h
                                * ({
                                    // Clamp  and then transform to be between 1. and 0.
                                    1. - ((*v).clamp(range.0, range.1) - range.0)
                                        / (range.1 - range.0)
                                }),
                        );
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
