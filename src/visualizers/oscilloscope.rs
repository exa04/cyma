use std::sync::{Arc, Mutex};

use vizia_plug::vizia::{prelude::*, vg};

use super::RangeModifiers;
use crate::accumulators::sample_delta;
use crate::{
    bus::Bus,
    utils::{RingBuffer, ValueScaling},
};

#[derive(Default, Copy, Clone)]
struct Sample {
    pub min: f32,
    pub max: f32,
}

const MAXED: Sample = Sample {
    min: f32::MAX,
    max: f32::MIN,
};

struct WaveformAccumulator {
    /// Maximum accumulator
    acc: Sample,
    size: usize,
    duration: f32,
    sample_rate: f32,
    /// The current time, counts down from sample_delta to 0
    t: f32,
    /// The decay time for the peak amplitude to halve.
    sample_delta: f32,
}

impl WaveformAccumulator {
    pub fn new(duration: f32) -> Self {
        Self {
            duration,
            acc: MAXED,
            size: 1,
            sample_delta: 1.0,
            sample_rate: 1.0,
            t: 0.0,
        }
    }

    fn update(self: &mut Self) {
        self.sample_delta = sample_delta(self.size, self.sample_rate, self.duration);
        self.t = 0.0;
    }

    #[inline]
    fn accumulate(&mut self, sample: f32) -> Option<Sample> {
        if sample > self.acc.max {
            self.acc.max = sample;
        }

        if sample < self.acc.min {
            self.acc.min = sample;
        }

        self.t += 1.0;

        if self.t > self.sample_delta {
            self.t -= self.sample_delta;
            let current = self.acc;
            self.acc = MAXED;

            Some(current)
        } else {
            None
        }
    }

    #[inline]
    fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
        self.update();
    }

    #[inline]
    fn set_size(&mut self, size: usize) {
        self.size = size;
        self.update();
    }
}

/// Displays the incoming signal as a waveform.
pub struct Oscilloscope<B: Bus<f32> + 'static> {
    dispatcher_handle: Arc<dyn Fn(<B as Bus<f32>>::O<'_>) + Send + Sync>,
    accumulator: Arc<Mutex<WaveformAccumulator>>,
    buffer: Arc<Mutex<RingBuffer<Sample>>>,
    range: (f32, f32),
    scaling: ValueScaling,
}

enum OscilloscopeEvents {
    UpdateRange((f32, f32)),
    UpdateScaling(ValueScaling),
}

impl<B: Bus<f32> + 'static> Oscilloscope<B> {
    /// Creates a new Oscilloscope displaying the last `duration` seconds of audio.
    pub fn new(
        cx: &mut Context,
        bus: Arc<B>,
        duration: f32,
        range: impl Res<(f32, f32)>,
        scaling: impl Res<ValueScaling>,
    ) -> Handle<Self> {
        let mut accumulator = WaveformAccumulator::new(duration);
        accumulator.set_sample_rate(bus.sample_rate());
        let accumulator = Arc::new(Mutex::new(accumulator));
        let accumulator_c = accumulator.clone();

        let buffer: Arc<Mutex<RingBuffer<Sample>>> = Default::default();
        let buffer_c = buffer.clone();

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
            dispatcher_handle,
            accumulator,
            buffer,
            range: range.get(cx),
            scaling: scaling.get(cx),
        }
        .build(cx, |_| {})
        .range(range)
        .scaling(scaling)
    }
}

impl<B: Bus<f32> + 'static> View for Oscilloscope<B> {
    fn element(&self) -> Option<&'static str> {
        Some("oscilloscope")
    }
    fn draw(&self, cx: &mut DrawContext, canvas: &vizia_plug::vizia::vg::Canvas) {
        let bounds = cx.bounds();

        let x = bounds.x;
        let y = bounds.y;
        let w = bounds.w;
        let h = bounds.h;

        let ring_buf = &mut self.buffer.lock().unwrap();

        {
            let width_ceil = w.ceil() as usize;
            if ring_buf.len() != width_ceil {
                ring_buf.resize(width_ceil);
                let mut acc = self.accumulator.lock().unwrap();
                acc.set_size(width_ceil);
            }
        }

        let len = ring_buf.len();

        let mut fill = vg::Path::new();

        // Local minima (bottom part of waveform)
        let mut py = self
            .scaling
            .value_to_normalized(ring_buf[0].min, self.range.0, self.range.1);
        fill.move_to((x, y + h * (1. - py) + 1.));
        for i in 1..len {
            py = self
                .scaling
                .value_to_normalized(ring_buf[i].min, self.range.0, self.range.1);

            fill.line_to((x + i as f32, y + h * (1. - py) + cx.scale_factor()));
        }

        // Local maxima (top part of waveform)
        py = self
            .scaling
            .value_to_normalized(ring_buf[len - 1].max, self.range.0, self.range.1);
        fill.line_to((x + w, y + h * (1. - py) + 1.));
        for i in 1..len {
            py =
                self.scaling
                    .value_to_normalized(ring_buf[len - i].max, self.range.0, self.range.1);

            fill.line_to((x + len as f32 - i as f32, y + h * (1. - py)));
        }

        fill.close();
        canvas.draw_path(
            &fill,
            &vg::Paint::new(Into::<vg::Color4f>::into(cx.font_color()), None)
                .set_anti_alias(true)
                .set_style(vg::PaintStyle::Fill),
        );
    }
    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        event.map(|e, _| match e {
            OscilloscopeEvents::UpdateRange(v) => self.range = *v,
            OscilloscopeEvents::UpdateScaling(v) => self.scaling = *v,
        });
    }
}

impl<'a, B: Bus<f32> + 'static> RangeModifiers for Handle<'a, Oscilloscope<B>> {
    fn range(mut self, range: impl Res<(f32, f32)>) -> Self {
        let e = self.entity();

        range.set_or_bind(self.context(), e, move |cx, r| {
            (*cx).emit_to(e, OscilloscopeEvents::UpdateRange(r.get(cx)));
        });

        self
    }
    fn scaling(mut self, scaling: impl Res<ValueScaling>) -> Self {
        let e = self.entity();

        scaling.set_or_bind(self.context(), e, move |cx, s| {
            (*cx).emit_to(e, OscilloscopeEvents::UpdateScaling(s.get(cx)));
        });

        self
    }
}
