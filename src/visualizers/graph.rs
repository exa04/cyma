use crate::utils::{ValueScaling, VisualizerBuffer};

use nih_plug_vizia::vizia::{
    binding::{Lens, LensExt, Res},
    context::{Context, DrawContext},
    vg,
    view::{Canvas, Handle, View},
};
use std::sync::{Arc, Mutex};

/// A real-time graph displaying information that is stored inside a buffer
pub struct Graph<L, I>
where
    L: Lens<Target = Arc<Mutex<I>>>,
    I: VisualizerBuffer<f32> + 'static,
{
    buffer: L,
    display_range: (f32, f32),
    scaling: ValueScaling,
    fill_from_top: bool,
}

impl<L, I> Graph<L, I>
where
    L: Lens<Target = Arc<Mutex<I>>>,
    I: VisualizerBuffer<f32, Output = f32> + 'static,
{
    pub fn new(
        cx: &mut Context,
        buffer: L,
        display_range: impl Res<(f32, f32)>,
        scaling: impl Res<ValueScaling>,
    ) -> Handle<Self> {
        Self {
            buffer,
            display_range: display_range.get_val(cx),
            scaling: scaling.get_val(cx),
            fill_from_top: false,
        }
        .build(cx, |_| {})
    }
}

impl<L, I> View for Graph<L, I>
where
    L: Lens<Target = Arc<Mutex<I>>>,
    I: VisualizerBuffer<f32, Output = f32> + 'static,
{
    fn element(&self) -> Option<&'static str> {
        Some("graph")
    }
    fn draw(&self, cx: &mut DrawContext, canvas: &mut Canvas) {
        let bounds = cx.bounds();

        let x = bounds.x;
        let y = bounds.y;
        let w = bounds.w;
        let h = bounds.h;

        let line_width = cx.scale_factor();

        let mut stroke = vg::Path::new();
        let binding = self.buffer.get(cx);
        let ring_buf = &(binding.lock().unwrap());

        let mut peak = self.scaling.value_to_normalized(
            ring_buf[0],
            self.display_range.0,
            self.display_range.1,
        );

        stroke.move_to(x, y + h * (1. - peak));

        for i in 1..ring_buf.len() {
            // Normalize peak value
            peak = self.scaling.value_to_normalized(
                ring_buf[i],
                self.display_range.0,
                self.display_range.1,
            );

            // Draw peak as a new point
            stroke.line_to(
                x + (w / ring_buf.len() as f32) * i as f32,
                y + h * (1. - peak),
            );
        }

        let mut fill = stroke.clone();

        if self.fill_from_top {
            fill.line_to(x + w, y);
            fill.line_to(x, y);
            fill.close();
        } else {
            fill.line_to(x + w, y + h);
            fill.line_to(x, y + h);
            fill.close();
        }

        canvas.fill_path(&fill, &vg::Paint::color(cx.background_color().into()));

        canvas.stroke_path(
            &stroke,
            &vg::Paint::color(cx.font_color().into()).with_line_width(line_width),
        );
    }
}

pub trait GraphModifiers {
    /// Allows for the grid to be filled from the top instead of the bottom.
    ///
    /// This is useful for certain graphs like gain reduction meters.
    fn should_fill_from_top(self, fill_from_top: bool) -> Self;
}

impl<'a, L, I> GraphModifiers for Handle<'a, Graph<L, I>>
where
    L: Lens<Target = Arc<Mutex<I>>>,
    I: VisualizerBuffer<f32, Output = f32> + 'static,
{
    fn should_fill_from_top(self, fill_from_top: bool) -> Self {
        self.modify(|graph| {
            graph.fill_from_top = fill_from_top;
        })
    }
}
