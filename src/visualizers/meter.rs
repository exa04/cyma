use std::sync::{Arc, Mutex};

use super::{FillFrom, FillModifiers, RangeModifiers};
use crate::bus::Bus;
use crate::accumulators::*;
use crate::utils::ValueScaling;
use nih_plug_vizia::vizia::{prelude::*, vg};

/// Displays some metric as a bar.
///
/// Can display different types of information about a signal:
///
///    - [`peak`](Self::peak) - Its peak amplitude
///    - [`minima`](Self::minima) - Its minimal amplitude
///    - [`rms`](Self::rms) - Its root mean squared level
///
/// It's also possible to define your own [`Accumulator`] in order to display some
/// other information about the incoming signal.
pub struct Meter<B: Bus<f32> + 'static, A: Accumulator + 'static> {
    dispatcher_handle: Arc<dyn Fn(<B as Bus<f32>>::O<'_>) + Send + Sync>,
    accumulator: Arc<Mutex<A>>,
    range: (f32, f32),
    scaling: ValueScaling,
    fill_from: FillFrom,
    orientation: Orientation,
}

impl<B: Bus<f32> + 'static, A: Accumulator + 'static> Meter<B, A> {
    /// Creates a new [`Meter`] which uses the provided [`Accumulator`].
    pub fn with_accumulator(
        cx: &mut Context,
        bus: Arc<B>,
        mut accumulator: A,
        range: impl Res<(f32, f32)>,
        scaling: impl Res<ValueScaling>,
        orientation: Orientation,
    ) -> Handle<Self> {
        accumulator.set_sample_rate(bus.sample_rate());
        accumulator.set_size(bus.sample_rate() as usize);

        let accumulator = Arc::new(Mutex::new(accumulator));
        let accumulator_c = accumulator.clone();

        let dispatcher_handle = bus.register_dispatcher(move |samples| {
            if let Ok(mut acc) = accumulator_c.lock() {
                for sample in samples {
                    let _ = acc.accumulate(*sample);
                }
            }
        });

        Self {
            dispatcher_handle,
            range: range.get_val(cx),
            scaling: scaling.get_val(cx),
            fill_from: FillFrom::Bottom,
            orientation,
            accumulator,
        }
        .build(cx, |_| {})
        .range(range)
        .scaling(scaling)
    }
}

enum MeterEvents {
    UpdateRange((f32, f32)),
    UpdateScaling(ValueScaling),
}

impl<B: Bus<f32> + 'static, A: Accumulator + 'static> View for Meter<B, A> {
    fn element(&self) -> Option<&'static str> {
        Some("meter")
    }
    fn draw(&self, cx: &mut DrawContext, canvas: &mut Canvas) {
        let bounds = cx.bounds();

        let x = bounds.x;
        let y = bounds.y;
        let w = bounds.w;
        let h = bounds.h;

        let sample = self.accumulator.lock().unwrap().prev();

        let level = self
            .scaling
            .value_to_normalized(sample, self.range.0, self.range.1);

        let mut path = vg::Path::new();
        match self.orientation {
            Orientation::Vertical => {
                path.move_to(x, y + h * (1. - level));
                path.line_to(x + w, y + h * (1. - level));

                let outline = path.clone();
                canvas.fill_path(&outline, &vg::Paint::color(cx.font_color().into()));

                let fill_from_n = match self.fill_from {
                    FillFrom::Top => 0.0,
                    FillFrom::Bottom => 1.0,
                    FillFrom::Value(val) => {
                        1.0 - ValueScaling::Linear.value_to_normalized(
                            val,
                            self.range.0,
                            self.range.1,
                        )
                    }
                };

                path.line_to(x + w, y + h * fill_from_n);
                path.line_to(x, y + h * fill_from_n);
                path.close();

                canvas.fill_path(&path, &vg::Paint::color(cx.background_color().into()));
            }
            Orientation::Horizontal => {
                path.move_to(x + w * level, y);
                path.line_to(x + w * level, y + h);

                let outline = path.clone();
                canvas.fill_path(&outline, &vg::Paint::color(cx.font_color().into()));

                let fill_from_n = match self.fill_from {
                    FillFrom::Top => 1.0,
                    FillFrom::Bottom => 0.0,
                    FillFrom::Value(val) => {
                        ValueScaling::Linear.value_to_normalized(val, self.range.0, self.range.1)
                    }
                };

                path.line_to(x + w * fill_from_n, y + h);
                path.line_to(x + w * fill_from_n, y);
                path.close();

                canvas.fill_path(&path, &vg::Paint::color(cx.background_color().into()));
            }
        };
    }
    fn event(&mut self, _cx: &mut EventContext, event: &mut Event) {
        event.map(|e, _| match e {
            MeterEvents::UpdateRange(v) => self.range = *v,
            MeterEvents::UpdateScaling(v) => self.scaling = *v,
        });
    }
}

impl<'a, B: Bus<f32> + 'static, A: Accumulator + 'static> FillModifiers
    for Handle<'a, Meter<B, A>>
{
    /// Allows for the meter to be filled from the maximum instead of the minimum value.
    ///
    /// This is useful for certain meters like gain reduction meters.
    fn fill_from_max(self) -> Self {
        self.modify(|meter| {
            meter.fill_from = FillFrom::Top;
        })
    }
    /// Allows for the meter to be filled from any desired level.
    ///
    /// This is useful for certain meters like gain reduction meters.
    fn fill_from_value(self, level: f32) -> Self {
        self.modify(|meter| {
            meter.fill_from = FillFrom::Value(level);
        })
    }
}

impl<'a, B: Bus<f32> + 'static, A: Accumulator + 'static> RangeModifiers
    for Handle<'a, Meter<B, A>>
{
    fn range(mut self, range: impl Res<(f32, f32)>) -> Self {
        let e = self.entity();

        range.set_or_bind(self.context(), e, move |cx, r| {
            (*cx).emit_to(e, MeterEvents::UpdateRange(r));
        });

        self
    }
    fn scaling(mut self, scaling: impl Res<ValueScaling>) -> Self {
        let e = self.entity();

        scaling.set_or_bind(self.context(), e, move |cx, s| {
            (*cx).emit_to(e, MeterEvents::UpdateScaling(s));
        });

        self
    }
}

impl<B: Bus<f32> + 'static> Meter<B, PeakAccumulator> {
    /// Creates a peak meter.
    ///
    /// # Example
    ///
    /// Peak meter with a 50ms-long decay for each peak.
    ///
    /// ```
    /// Meter::peak(
    ///     cx,
    ///     bus.clone(),
    ///     50.0,
    ///     (-32.0, 8.0),
    ///     ValueScaling::Decibels,
    ///     Orientation::Vertical,
    /// )
    /// .color(Color::rgba(255, 255, 255, 60))
    /// .background_color(Color::rgba(255, 255, 255, 30));
    /// ```
    pub fn peak(
        cx: &mut Context,
        bus: Arc<B>,
        decay: f32,
        range: impl Res<(f32, f32)> + Clone,
        scaling: impl Res<ValueScaling> + Clone,
        orientation: Orientation,
    ) -> Handle<Self> {
        Self::with_accumulator(
            cx,
            bus,
            PeakAccumulator::new(1.0, decay),
            range,
            scaling,
            orientation,
        )
    }
}
impl<B: Bus<f32> + 'static> Meter<B, MinimumAccumulator> {
    /// Creates a peak meter.
    ///
    /// # Example
    ///
    /// Peak meter with a 50ms-long decay for each peak.
    ///
    /// This may be useful for gain reduction meters.
    ///
    /// ```
    /// Meter::minima(
    ///     cx,
    ///     bus.clone(),
    ///     50.0,
    ///     (-32.0, 8.0),
    ///     ValueScaling::Decibels,
    ///     Orientation::Vertical,
    /// )
    /// .color(Color::rgba(255, 255, 255, 60))
    /// ```
    pub fn minima(
        cx: &mut Context,
        bus: Arc<B>,
        decay: f32,
        range: impl Res<(f32, f32)> + Clone,
        scaling: impl Res<ValueScaling> + Clone,
        orientation: Orientation,
    ) -> Handle<Self> {
        Self::with_accumulator(
            cx,
            bus,
            MinimumAccumulator::new(1.0, decay),
            range,
            scaling,
            orientation,
        )
    }
}
impl<B: Bus<f32> + 'static> Meter<B, RMSAccumulator> {
    /// Creates an RMS meter.
    ///
    /// # Example
    ///
    /// 10-second RMS meter showing the RMS level over a 250 ms long window.
    ///
    /// ```
    /// Graph::rms(
    ///     cx,
    ///     bus.clone(),
    ///     250.0,
    ///     (-32.0, 8.0),
    ///     ValueScaling::Decibels,
    ///     Orientation::Vertical,
    /// )
    /// .color(Color::rgba(255, 255, 255, 60))
    /// .background_color(Color::rgba(255, 255, 255, 30));
    /// ```
    pub fn rms(
        cx: &mut Context,
        bus: Arc<B>,
        window_size: f32,
        range: impl Res<(f32, f32)> + Clone,
        scaling: impl Res<ValueScaling> + Clone,
        orientation: Orientation,
    ) -> Handle<Self> {
        Self::with_accumulator(
            cx,
            bus,
            RMSAccumulator::new(1.0, window_size),
            range,
            scaling,
            orientation,
        )
    }
}
