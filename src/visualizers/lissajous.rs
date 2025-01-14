use crate::{bus::Bus, utils::RingBuffer};

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

lazy_static! {
    static ref TRANSLATE_SIN: f32 = (PI / 4.).sin();
    static ref TRANSLATE_COS: f32 = (PI / 4.).cos();
}

type Sample = [f32; 2];

// L: Lens<Target = Arc<Mutex<RingBuffer<(f32, f32)>>>>,

/// Lissajous for stereo audio data.
///
/// The further points are from the horizontal middle, the more stereo your signal
/// is. If they are closer to the top or bottom, your signal has a positive or a
/// negative offset, respectively. If the lissajous shows a straight horizontal
/// line, your channels are fully out of phase. If it skews either way diagonally,
/// your signal is more present on the left or right side, respectively.
///
/// For more information about lissajous curves, check out the
/// [Wikipedia entry](https://en.wikipedia.org/wiki/Lissajous_curve) on them.
pub struct Lissajous<B: Bus<Sample> + 'static> {
    buffer: Arc<Mutex<RingBuffer<Sample>>>,
    dispatcher: Arc<dyn Fn(<B as Bus<[f32; 2]>>::O<'_>) + Send + Sync>,
}

impl<B: Bus<Sample> + 'static> Lissajous<B> {
    pub fn new(cx: &mut Context, bus: Arc<B>, duration: usize) -> Handle<Self> {
        let buffer = Arc::new(Mutex::new(RingBuffer::<Sample>::new(duration)));
        let buffer_c = buffer.clone();

        let dispatcher = bus.register_dispatcher(move |samples| {
            if let Ok(mut buffer) = buffer_c.lock() {
                for sample in samples {
                    buffer.enqueue(*sample);
                }
            }
        });

        Self { buffer, dispatcher }.build(cx, |_| {})
    }
}

impl<B: Bus<Sample> + 'static> View for Lissajous<B> {
    fn element(&self) -> Option<&'static str> {
        None
    }
    fn draw(&self, cx: &mut DrawContext, canvas: &mut Canvas) {
        let bounds = cx.bounds();

        let x = bounds.x;
        let y = bounds.y;
        let w = bounds.w;
        let h = bounds.h;

        let ring_buf = &(self.buffer.lock().unwrap());

        canvas.fill_path(
            &{
                let mut dots = vg::Path::new();

                for i in 0..ring_buf.len() {
                    let left = ring_buf[i][0].clamp(-1., 1.);
                    let right = ring_buf[i][1].clamp(-1., 1.);

                    let dot_x = left * *TRANSLATE_COS - right * *TRANSLATE_SIN;
                    let dot_y = left * *TRANSLATE_SIN + right * *TRANSLATE_COS;

                    dots.rect(
                        x + w / 2. - dot_x * w / PI,
                        y + h / 2. - dot_y * h / PI,
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

/// A diamond-shaped grid that can serve as a backdrop for a [`Lissajous`]
pub struct LissajousGrid {}

impl LissajousGrid {
    pub fn new(cx: &mut Context) -> Handle<Self> {
        Self {}.build(cx, |_| {})
    }
}

impl View for LissajousGrid {
    fn element(&self) -> Option<&'static str> {
        Some("lissajous")
    }
    fn draw(&self, cx: &mut DrawContext, canvas: &mut Canvas) {
        let bounds = cx.bounds();

        let x = bounds.x;
        let y = bounds.y;
        let w = bounds.w;
        let h = bounds.h;

        let mut path = vg::Path::new();

        // Diamond shape
        path.move_to(x + w / 2., y);
        path.line_to(x + w, y + h / 2.);
        path.line_to(x + w / 2., y + h);
        path.line_to(x, y + h / 2.);
        path.close();

        canvas.fill_path(
            &(path.clone()),
            &vg::Paint::color(cx.background_color().into()),
        );

        // Vertical line
        path.move_to(x, y + h / 2.);
        path.line_to(x + w, y + h / 2.);
        path.close();

        // Horizontal line
        path.move_to(x + w / 2., y);
        path.line_to(x + w / 2., y + h);
        path.close();

        // Diagonal line (Left channel)
        path.move_to(x + w * 0.25, y + h * 0.25);
        path.line_to(x + w * 0.75, y + h * 0.75);
        path.close();

        // Diagonal line (Right channel)
        path.move_to(x + w * 0.75, y + h * 0.25);
        path.line_to(x + w * 0.25, y + h * 0.75);
        path.close();

        canvas.stroke_path(&path, &vg::Paint::color(cx.font_color().into()));
    }
}
