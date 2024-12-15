use super::{FillFrom, FillModifiers, RangeModifiers};
use crate::prelude::MonoChannel;
use crate::utils::{accumulators::ValueAccumulator, MonoChannelConsumer, RingBuffer, ValueScaling};
use nih_plug::prelude::AtomicF32;
use nih_plug_vizia::vizia::{prelude::*, vg};
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};

pub struct Graph<A: ValueAccumulator + 'static> {
    consumer: Arc<Mutex<MonoChannelConsumer>>,
    buffer: Arc<Mutex<RingBuffer<f32>>>,
    range: (f32, f32),
    scaling: ValueScaling,
    fill_from: FillFrom,
    accumulator: Arc<Mutex<A>>,
}

enum GraphEvents {
    UpdateRange((f32, f32)),
    UpdateScaling(ValueScaling),
}

impl<A: ValueAccumulator + 'static> Graph<A> {
    pub fn new(
        cx: &mut Context,
        mut accumulator: A,
        range: impl Res<(f32, f32)> + Clone,
        scaling: impl Res<ValueScaling> + Clone,
        channel: MonoChannel,
    ) -> Handle<Self> {
        let consumer = channel.get_consumer();
        accumulator.set_sample_rate(consumer.get_sample_rate());

        Self {
            consumer: Arc::new(Mutex::new(consumer)),
            buffer: Default::default(),
            range: range.get_val(cx),
            scaling: scaling.get_val(cx),
            fill_from: FillFrom::Bottom,
            accumulator: Arc::new(Mutex::new(accumulator)),
        }
        .build(cx, |_| {})
        .range(range)
        .scaling(scaling)
    }
}

impl<A: ValueAccumulator + 'static> View for Graph<A> {
    fn element(&self) -> Option<&'static str> {
        Some("graph")
    }
    fn event(&mut self, _cx: &mut EventContext, event: &mut Event) {
        event.map(|e, _| match e {
            GraphEvents::UpdateRange(v) => self.range = *v,
            GraphEvents::UpdateScaling(s) => self.scaling = *s,
        });
    }
    fn draw(&self, cx: &mut DrawContext, canvas: &mut Canvas) {
        let bounds = cx.bounds();

        let x = bounds.x;
        let y = bounds.y;
        let w = bounds.w;
        let h = bounds.h;

        let line_width = cx.scale_factor();

        // Update buffer

        let ring_buf = &mut (self.buffer.lock().unwrap());

        {
            let mut acc = self.accumulator.lock().unwrap();

            let width_ceil = w.ceil() as usize;
            if ring_buf.len() != width_ceil {
                ring_buf.resize(width_ceil);
                acc.set_size(width_ceil);
            }

            let mut consumer = self.consumer.lock().unwrap();

            while let Some(sample) = consumer.receive() {
                if let Some(sample) = acc.accumulate(sample) {
                    ring_buf.enqueue(sample);
                }
            }
        }

        let mut peak = self
            .scaling
            .value_to_normalized(ring_buf[0], self.range.0, self.range.1);

        // Draw

        let mut stroke = vg::Path::new();

        stroke.move_to(x, y + h * (1. - peak));

        for i in 1..ring_buf.len() {
            // Normalize peak value
            peak = self
                .scaling
                .value_to_normalized(ring_buf[i], self.range.0, self.range.1);

            // Draw peak as a new point
            stroke.line_to(x + i as f32, y + h * (1. - peak));
        }

        let mut fill = stroke.clone();
        let fill_from_n = match self.fill_from {
            FillFrom::Top => 0.0,
            FillFrom::Bottom => 1.0,
            FillFrom::Value(val) => {
                1.0 - ValueScaling::Linear.value_to_normalized(val, self.range.0, self.range.1)
            }
        };

        fill.line_to(x + w, y + h * fill_from_n);
        fill.line_to(x, y + h * fill_from_n);
        fill.close();

        canvas.fill_path(&fill, &vg::Paint::color(cx.background_color().into()));

        canvas.stroke_path(
            &stroke,
            &vg::Paint::color(cx.font_color().into()).with_line_width(line_width),
        );
    }
}

impl<'a, A: ValueAccumulator + 'static> FillModifiers for Handle<'a, Graph<A>> {
    /// Allows for the graph to be filled from the top instead of the bottom.
    ///
    /// This is useful for certain graphs like gain reduction meters.
    ///
    /// # Example
    ///
    /// Here's a gain reduction graph, which you could overlay on top of a peak graph.
    ///
    /// Here, `gain_mult` could be a [`MinimaBuffer`](crate::utils::MinimaBuffer).
    ///
    /// ```
    /// Graph::new(cx, Data::gain_mult, (-32.0, 8.0), ValueScaling::Decibels)
    ///     .fill_from_max()
    ///     .color(Color::rgba(255, 0, 0, 160))
    ///     .background_color(Color::rgba(255, 0, 0, 60));
    /// ```
    fn fill_from_max(self) -> Self {
        self.modify(|graph| {
            graph.fill_from = FillFrom::Top;
        })
    }
    /// Allows for the graph to be filled from any desired level.
    ///
    /// This is useful for certain graphs like gain reduction meters.
    ///
    /// # Example
    ///
    /// Here's a gain reduction graph, which you could overlay on top of a peak graph.
    ///
    /// Here, `gain_mult` could be a [`MinimaBuffer`](crate::utils::MinimaBuffer).
    ///
    /// ```
    /// Graph::new(cx, Data::gain_mult, (-32.0, 6.0), ValueScaling::Decibels)
    ///     .fill_from(0.0) // Fills the graph from 0.0dB downwards
    ///     .color(Color::rgba(255, 0, 0, 160))
    ///     .background_color(Color::rgba(255, 0, 0, 60));
    /// ```
    fn fill_from_value(self, level: f32) -> Self {
        self.modify(|graph| {
            graph.fill_from = FillFrom::Value(level);
        })
    }
}

impl<'a, A: ValueAccumulator + 'static> RangeModifiers for Handle<'a, Graph<A>> {
    fn range(mut self, range: impl Res<(f32, f32)>) -> Self {
        let e = self.entity();

        range.set_or_bind(self.context(), e, move |cx, r| {
            (*cx).emit_to(e, GraphEvents::UpdateRange(r.clone()));
        });

        self
    }
    fn scaling(mut self, scaling: impl Res<ValueScaling>) -> Self {
        let e = self.entity();

        scaling.set_or_bind(self.context(), e, move |cx, s| {
            (*cx).emit_to(e, GraphEvents::UpdateScaling(s.clone()))
        });

        self
    }
}
