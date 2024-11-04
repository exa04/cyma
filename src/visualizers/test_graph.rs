use std::collections::VecDeque;
use std::fmt::Pointer;
use std::sync::Arc;
use arc_swap::ArcSwap;
use nih_plug_vizia::vizia::prelude::*;
use nih_plug_vizia::vizia::vg;
use realfft::num_traits::float::FloatCore;
use crate::utils::{Outlet, OutletConsumer, PeakBuffer, RingBuffer, VisualizerBuffer};

// Naively downsampled graph
pub struct TestGraph<O>
where
    O: Outlet,
{
    consumer: O::Consumer,
    buffer: ArcSwap<PeakBuffer>,
}

impl<O> TestGraph<O>
where
    O: Outlet + 'static,
{
    pub fn new(cx: &mut Context, outlet: impl Lens<Target = O>) -> Handle<Self> {
        Self {
            consumer: outlet.get(cx).get_consumer(),
            buffer: ArcSwap::new(PeakBuffer::new(0, 0.0, 0.0).into()),
        }
            .build(cx, |_| {})
    }
}

impl<O> View for TestGraph<O>
where
    O: Outlet + 'static,
{
    fn draw(&self, cx: &mut DrawContext, canvas: &mut Canvas) {
        let mut buffer = (**self.buffer.load()).clone();

        let bounds = cx.bounds();

        let x = bounds.x;
        let y = bounds.y;
        let w = bounds.w;
        let h = bounds.h;

        let wceil = w.ceil() as usize;

        if wceil != buffer.len() {
            buffer = PeakBuffer::new(wceil, 10.0, 50.0);
            buffer.set_sample_rate(44_100.0);
        }

        let line_width = cx.scale_factor();

        let mut stroke = vg::Path::new();

        self.consumer.receive().iter().for_each(|x| buffer.enqueue(*x));
        let buffer = Arc::new(buffer);
        self.buffer.store(buffer.clone());

        stroke.move_to(x, y + h * (1.0 - buffer[0]));

        for i in 1..buffer.len() {
            stroke.line_to(x + i as f32, y + h * (1.0 - buffer[i]));
        }

        let mut fill = stroke.clone();

        fill.line_to(x + w, y +h);
        fill.line_to(x, y + h);

        canvas.fill_path(&fill, &vg::Paint::color(cx.background_color().into()));
        canvas.stroke_path(&stroke, &vg::Paint::color(cx.font_color().into()));
    }
}