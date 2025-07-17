use super::{FillFrom, FillModifiers, RangeModifiers};
use crate::accumulators::*;
use crate::bus::Bus;
use crate::utils::{RingBuffer, ValueScaling};
use std::sync::{Arc, Mutex};
use vizia_plug::vizia::{prelude::*, vg};

/// A graph visualizer plotting a value over time.
///
/// Can display different types of information about a signal:
///
///    - [`peak`](Self::peak) - Its peak amplitude
///    - [`minima`](Self::minima) - Its minimal amplitude
///    - [`rms`](Self::rms) - Its root mean squared level
///
/// It's also possible to define your own [`Accumulator`] in order to display some
/// other information about the incoming signal.
pub struct Graph<B: Bus<f32> + 'static, A: Accumulator + 'static> {
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
    /// Creates a new [`Graph`] which uses the provided [`Accumulator`].
    pub fn with_accumulator(
        cx: &mut Context,
        bus: Arc<B>,
        mut accumulator: A,
        range: impl Res<(f32, f32)> + Clone,
        scaling: impl Res<ValueScaling> + Clone,
    ) -> Handle<Self> {
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
            buffer,
            range: range.get(cx),
            scaling: scaling.get(cx),
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
    fn draw(&self, cx: &mut DrawContext, canvas: &vizia_plug::vizia::vg::Canvas) {
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
        }

        if ring_buf.len() == 0 {
            return;
        }

        let mut peak = self
            .scaling
            .value_to_normalized(ring_buf[0], self.range.0, self.range.1);

        // Draw

        let mut stroke = vg::Path::new();

        stroke.move_to((x, y + h * (1. - peak)));

        for i in 1..ring_buf.len() {
            // Normalize peak value
            peak = self
                .scaling
                .value_to_normalized(ring_buf[i], self.range.0, self.range.1);

            // Draw peak as a new point
            stroke.line_to((x + i as f32, y + h * (1. - peak)));
        }

        let mut fill = stroke.clone();
        let fill_from_n = match self.fill_from {
            FillFrom::Top => 0.0,
            FillFrom::Bottom => 1.0,
            FillFrom::Value(val) => {
                1.0 - ValueScaling::Linear.value_to_normalized(val, self.range.0, self.range.1)
            }
        };

        fill.line_to((x + w, y + h * fill_from_n));
        fill.line_to((x, y + h * fill_from_n));
        fill.close();

        canvas.draw_path(
            &fill,
            &vg::Paint::new(Into::<vg::Color4f>::into(cx.background_color()), None)
                .set_style(vg::PaintStyle::Fill)
                .set_anti_alias(true),
        );

        canvas.draw_path(
            &stroke,
            &vg::Paint::new(Into::<vg::Color4f>::into(cx.font_color()), None)
                .set_style(vg::PaintStyle::Stroke)
                .set_stroke_width(line_width)
                .set_anti_alias(true),
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
            (*cx).emit_to(e, GraphEvents::UpdateRange(r.get(cx)));
        });

        self
    }
    fn scaling(mut self, scaling: impl Res<ValueScaling>) -> Self {
        let e = self.entity();

        scaling.set_or_bind(self.context(), e, move |cx, s| {
            (*cx).emit_to(e, GraphEvents::UpdateScaling(s.get(cx)))
        });

        self
    }
}

impl<B: Bus<f32> + 'static> Graph<B, PeakAccumulator> {
    /// Creates a peak graph.
    ///
    /// # Example
    ///
    /// 10-second peak graph with a 50ms-long decay for each peak.
    ///
    /// ```
    /// Graph::peak(
    ///     cx,
    ///     bus.clone(),
    ///     10.0,
    ///     50.0,
    ///     (-32.0, 8.0),
    ///     ValueScaling::Decibels,
    /// )
    /// .color(Color::rgba(255, 255, 255, 60))
    /// .background_color(Color::rgba(255, 255, 255, 30));
    /// ```
    pub fn peak(
        cx: &mut Context,
        bus: Arc<B>,
        duration: f32,
        decay: f32,
        range: impl Res<(f32, f32)> + Clone,
        scaling: impl Res<ValueScaling> + Clone,
    ) -> Handle<Self> {
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
    /// Creates a minima graph.
    ///
    /// This may be useful for gain reduction graphs.
    ///
    /// ## Example
    ///
    /// 50-second minima graph with a 50ms-long decay for each minimum.
    ///
    /// ```
    /// Graph::minima(
    ///     cx,
    ///     gain_reduction_bus.clone(),
    ///     10.0,
    ///     50.0,
    ///     (-32.0, 8.0),
    ///     ValueScaling::Decibels,
    /// )
    /// .color(Color::rgba(255, 255, 255, 60))
    /// .background_color(Color::rgba(255, 255, 255, 30));
    /// ```
    pub fn minima(
        cx: &mut Context,
        bus: Arc<B>,
        duration: f32,
        decay: f32,
        range: impl Res<(f32, f32)> + Clone,
        scaling: impl Res<ValueScaling> + Clone,
    ) -> Handle<Self> {
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
    /// Creates a graph showing the root mean squared level over time.
    ///
    /// ## Example
    ///
    /// 10-second RMS graph showing the RMS level over a 250 ms long window.
    ///
    /// ```
    /// Graph::rms(
    ///     cx,
    ///     bus.clone(),
    ///     10.0,
    ///     250.0,
    ///     (-32.0, 8.0),
    ///     ValueScaling::Decibels,
    /// )
    /// .color(Color::rgba(255, 92, 92, 128));
    /// ```
    pub fn rms(
        cx: &mut Context,
        bus: Arc<B>,
        duration: f32,
        window_size: f32,
        range: impl Res<(f32, f32)> + Clone,
        scaling: impl Res<ValueScaling> + Clone,
    ) -> Handle<Self> {
        Self::with_accumulator(
            cx,
            bus,
            RMSAccumulator::new(duration, window_size),
            range,
            scaling,
        )
    }
}
