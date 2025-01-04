use super::{FillFrom, FillModifiers, RangeModifiers};
use crate::bus::Bus;
use crate::utils::accumulators::{MinimumAccumulator, PeakAccumulator, RMSAccumulator};
use crate::utils::{accumulators::Accumulator, RingBuffer, ValueScaling};
use nih_plug_vizia::vizia::{prelude::*, vg};
use std::sync::{Arc, Mutex};

pub struct Graph<B: Bus<f32> + 'static, A: Accumulator + 'static> {
    bus: Arc<B>,
    buffer: Arc<Mutex<RingBuffer<f32>>>,
    range: (f32, f32),
    scaling: ValueScaling,
    fill_from: FillFrom,
    accumulator: Arc<Mutex<A>>,
    dispatcher_handle: Arc<dyn Fn(<B as Bus<f32>>::O<'_>) + Sync + Send + 'static>,
}

enum GraphEvents {
    UpdateRange((f32, f32)),
    UpdateScaling(ValueScaling),
}

impl<B: Bus<f32> + 'static, A: Accumulator + 'static> Graph<B, A> {
    pub fn with_accumulator<L>(
        cx: &mut Context,
        bus: L,
        mut accumulator: A,
        range: impl Res<(f32, f32)> + Clone,
        scaling: impl Res<ValueScaling> + Clone,
    ) -> Handle<Self>
    where
        L: Lens<Target = Arc<B>>,
    {
        let bus = bus.get(cx);

        accumulator.set_sample_rate(bus.sample_rate());

        let buffer: Arc<Mutex<RingBuffer<f32>>> = Default::default();
        let buffer_c = buffer.clone();

        let accumulator = Arc::new(Mutex::new(accumulator));
        let accumulator_c = accumulator.clone();

        let dispatcher_handle = bus.register_dispatcher(move |samples| {
            if let (Ok(mut buf), Ok(mut acc)) = (buffer_c.lock(), accumulator_c.lock()) {
                for sample in samples {
                    if let Some(sample) = acc.accumulate(*sample) {
                        buf.enqueue(sample);
                    }
                }
            }
        });

        Self {
            bus,
            buffer,
            range: range.get_val(cx),
            scaling: scaling.get_val(cx),
            fill_from: FillFrom::Bottom,
            accumulator,
            dispatcher_handle,
        }
        .build(cx, |_| {})
        .range(range)
        .scaling(scaling)
    }
}
impl<B: Bus<f32>, A: Accumulator + 'static> View for Graph<B, A> {
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

        self.bus.update();

        let ring_buf = &mut (self.buffer.lock().unwrap());

        {
            let mut acc = self.accumulator.lock().unwrap();

            let width_ceil = w.ceil() as usize;
            if ring_buf.len() != width_ceil {
                ring_buf.resize(width_ceil);
                acc.set_size(width_ceil);
            }
        }

        if ring_buf.len() == 0 {
            return;
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

impl<'a, B: Bus<f32> + 'static, A: Accumulator + 'static> FillModifiers
    for Handle<'a, Graph<B, A>>
{
    fn fill_from_max(self) -> Self {
        self.modify(|graph| {
            graph.fill_from = FillFrom::Top;
        })
    }
    fn fill_from_value(self, level: f32) -> Self {
        self.modify(|graph| {
            graph.fill_from = FillFrom::Value(level);
        })
    }
}

impl<'a, B: Bus<f32> + 'static, A: Accumulator + 'static> RangeModifiers
    for Handle<'a, Graph<B, A>>
{
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

impl<B: Bus<f32> + 'static> Graph<B, PeakAccumulator> {
    pub fn peak<L>(
        cx: &mut Context,
        bus: L,
        duration: f32,
        decay: f32,
        range: impl Res<(f32, f32)> + Clone,
        scaling: impl Res<ValueScaling> + Clone,
    ) -> Handle<Self>
    where
        L: Lens<Target = Arc<B>>,
    {
        Self::with_accumulator(
            cx,
            bus,
            PeakAccumulator::new(duration, decay),
            range,
            scaling,
        )
    }
}
impl<B: Bus<f32> + 'static> Graph<B, MinimumAccumulator> {
    pub fn minima<L>(
        cx: &mut Context,
        bus: L,
        duration: f32,
        decay: f32,
        range: impl Res<(f32, f32)> + Clone,
        scaling: impl Res<ValueScaling> + Clone,
    ) -> Handle<Self>
    where
        L: Lens<Target = Arc<B>>,
    {
        Self::with_accumulator(
            cx,
            bus,
            MinimumAccumulator::new(duration, decay),
            range,
            scaling,
        )
    }
}
impl<B: Bus<f32> + 'static> Graph<B, RMSAccumulator> {
    pub fn rms<L>(
        cx: &mut Context,
        bus: L,
        duration: f32,
        window_size: f32,
        range: impl Res<(f32, f32)> + Clone,
        scaling: impl Res<ValueScaling> + Clone,
    ) -> Handle<Self>
    where
        L: Lens<Target = Arc<B>>,
    {
        Self::with_accumulator(
            cx,
            bus,
            RMSAccumulator::new(duration, window_size),
            range,
            scaling,
        )
    }
}
