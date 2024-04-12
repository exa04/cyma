use crate::utils::RingBuffer;

use lazy_static::lazy_static;
use nih_plug_vizia::vizia::{
    binding::{Lens, LensExt},
    context::{Context, DrawContext},
    vg,
    view::{Canvas, Handle, View},
};
use std::{
    f32::consts::PI,
    sync::{Arc, Mutex},
};

// These will be used to rotate the lissajous such that it is a straight
// vertical line for mono data and a horizontal line for fully stereo data.
lazy_static! {
    static ref TRANSLATE_SIN: f32 = (PI / 4.).sin();
    static ref TRANSLATE_COS: f32 = (PI / 4.).cos();
}

pub struct Lissajous<L>
where
    L: Lens<Target = Arc<Mutex<RingBuffer<(f32, f32)>>>>,
{
    buffer: L,
}

impl<L> Lissajous<L>
where
    L: Lens<Target = Arc<Mutex<RingBuffer<(f32, f32)>>>>,
{
    pub fn new(cx: &mut Context, buffer: L) -> Handle<Self> {
        Self { buffer }.build(cx, |_| {})
    }
}

impl<L> View for Lissajous<L>
where
    L: Lens<Target = Arc<Mutex<RingBuffer<(f32, f32)>>>>,
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

        canvas.fill_path(
            &{
                let mut dots = vg::Path::new();

                for i in 0..ring_buf.len() {
                    let left = ring_buf[i].0.clamp(-1., 1.);
                    let right = ring_buf[i].1.clamp(-1., 1.);

                    let dot_x = left * *TRANSLATE_COS - right * *TRANSLATE_SIN;
                    let dot_y = left * *TRANSLATE_SIN + right * *TRANSLATE_COS;

                    dots.rect(
                        x + w / 2. + dot_x * w / PI,
                        y + h / 2. + dot_y * h / PI,
                        1f32,
                        1f32,
                    );
                }

                dots
            },
            &vg::Paint::color(cx.font_color().into()),
        );
    }
}
