use std::sync::{Arc, Mutex};

use nih_plug_vizia::vizia::{prelude::*, vg};

/// Displays a static waveform. For displaying frequently updating waveform
/// data, use an [`Oscilloscope`] instead.
pub struct Waveform<V>
where
    V: Lens<Target = Arc<Mutex<Vec<f32>>>>,
{
    data: V,
}

impl<V> Waveform<V>
where
    V: Lens<Target = Arc<Mutex<Vec<f32>>>>,
{
    pub fn new(cx: &mut Context, data: V) -> Handle<Self> {
        Self { data }.build(cx, |_| {})
    }
}

impl<V> View for Waveform<V>
where
    V: Lens<Target = Arc<Mutex<Vec<f32>>>>,
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

        // Waveform
        canvas.stroke_path(
            &{
                let mut path = vg::Path::new();
                let binding = self.data.get(cx);
                let ring_buf = binding.lock().unwrap();

                path.move_to(x, y + (h / 2.) * (1. - ring_buf[0].clamp(-1., 1.)));

                let mut i = 0.;
                for v in (&ring_buf).iter() {
                    path.line_to(
                        x + (w / ring_buf.len() as f32) * i,
                        y + (h / 2.) * (1. - v.clamp(-1., 1.)),
                    );
                    i += 1.;
                }
                path
            },
            &vg::Paint::color(cx.font_color().into())
                .with_line_width(cx.scale_factor() * cx.outline_width()),
        );
    }
}
