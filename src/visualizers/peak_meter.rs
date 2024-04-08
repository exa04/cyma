use std::sync::{Arc, Mutex};

use nih_plug_vizia::vizia::{prelude::*, vg};

use crate::utils::PeakBuffer;
use crate::utils::ValueScaling;

pub struct PeakMeter<B>
where
    B: Lens<Target = Arc<Mutex<PeakBuffer<f32>>>>,
{
    buffer: B,
    display_range: (f32, f32),
    scaling: ValueScaling,
    orientation: Orientation,
}

impl<B> PeakMeter<B>
where
    B: Lens<Target = Arc<Mutex<PeakBuffer<f32>>>>,
{
    pub fn new(
        cx: &mut Context,
        buffer: B,
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

impl<B> View for PeakMeter<B>
where
    B: Lens<Target = Arc<Mutex<PeakBuffer<f32>>>>,
{
    fn element(&self) -> Option<&'static str> {
        None
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
