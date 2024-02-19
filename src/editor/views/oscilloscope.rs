use std::sync::{Arc, Mutex};

use nih_plug_vizia::vizia::{prelude::*, vg};

use crate::utils::PeakWaveformRingBuffer;

pub struct Oscilloscope<B, const BUFFER_SIZE: usize>
where
    B: Lens<Target = Arc<Mutex<PeakWaveformRingBuffer<f32, BUFFER_SIZE>>>>,
{
    buffer: B,
}

impl<B, const BUFFER_SIZE: usize> Oscilloscope<B, BUFFER_SIZE>
where
    B: Lens<Target = Arc<Mutex<PeakWaveformRingBuffer<f32, BUFFER_SIZE>>>>,
{
    pub fn new(cx: &mut Context, buffer: B) -> Handle<Self> {
        Self { buffer }.build(cx, |cx| {})
    }
}

impl<B, const BUFFER_SIZE: usize> View for Oscilloscope<B, BUFFER_SIZE>
where
    B: Lens<Target = Arc<Mutex<PeakWaveformRingBuffer<f32, BUFFER_SIZE>>>>,
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

        // Background
        canvas.fill_path(
            &{
                let mut path = vg::Path::new();
                path.move_to(x, y);
                path.line_to(x + w, y);
                path.line_to(x + w, y + h);
                path.line_to(x, y + h);
                path.close();
                path
            },
            &vg::Paint::color(cx.background_color().into()),
        );

        // Waveform
        canvas.fill_path(
            &{
                let mut path = vg::Path::new();
                let binding = self.buffer.get(cx);
                let ring_buf = &(binding.lock().unwrap()).ring_buffer;

                path.move_to(x, y + h / 2.);

                let mut i = 0.;
                for v in ring_buf.into_iter() {
                    path.line_to(
                        x + (w / BUFFER_SIZE as f32) * i,
                        y + (h / 2.) * (1. - v.0) + 1.,
                    );
                    i += 1.;
                }
                for v in ring_buf.into_iter().rev() {
                    i -= 1.;
                    path.line_to(
                        x + (w / BUFFER_SIZE as f32) * i,
                        y + (h / 2.) * (1. - v.1) + 1.,
                    );
                }
                path.close();
                path
            },
            &vg::Paint::color(cx.font_color().into()).with_line_width(line_width),
        );
    }
}
