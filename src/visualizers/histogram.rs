use crate::bus::Bus;
use crate::utils::ValueScaling;
use nih_plug::prelude::AtomicF32;
use nih_plug_vizia::vizia::{prelude::*, vg};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

struct HistogramState {
    data: [AtomicF32; 2048],
    edges: [AtomicF32; 2047],

    sample_rate: f32,
    decay: f32,

    size: AtomicUsize,
    decay_weight: AtomicF32,
}

/// A histogram plot of the most frequent levels in a signal.
pub struct Histogram<B: Bus<f32> + 'static> {
    dispatcher_handle: Arc<dyn Fn(<B as Bus<f32>>::O<'_>) + Send + Sync>,
    state: Arc<HistogramState>,
    range: (f32, f32),
    scaling: ValueScaling,
}

impl<B: Bus<f32> + 'static> Histogram<B> {
    /// Creates a new [`Histogram`].
    pub fn new(
        cx: &mut Context,
        bus: Arc<B>,
        decay: f32,
        range: (f32, f32),
        scaling: ValueScaling,
    ) -> Handle<Self> {
        let state: Arc<_> = HistogramState {
            data: [0f32; 2048].map(|x| x.into()),
            edges: [0f32; 2047].map(|x| x.into()),
            sample_rate: bus.sample_rate(),
            decay,
            size: 1.into(),
            decay_weight: 0.0.into(),
        }
        .into();

        let state_c = state.clone();

        let dispatcher_handle = bus.register_dispatcher(move |samples| {
            let decay_weight = state_c.decay_weight.load(Ordering::Relaxed);
            let total_decay_weight = decay_weight.powi(samples.len() as i32);

            for i in 0..state_c.size.load(Ordering::Relaxed) - 1 {
                state_c.data[i]
                    .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |sample| {
                        Some(sample * total_decay_weight)
                    })
                    .unwrap();
            }

            for sample in samples {
                state_c.data[{
                    let value = sample.abs();
                    if value < state_c.edges[0].load(Ordering::Relaxed) {
                        0
                    } else {
                        let size = state_c.size.load(Ordering::Relaxed);

                        // Check if the value is larger than the last edge
                        if value > state_c.edges[size - 1].load(Ordering::Relaxed) {
                            state_c.edges.len()
                        } else {
                            // Binary search to find the bin for the given value
                            let mut left = 0;
                            let mut right = size - 1;

                            while left <= right {
                                let mid = left + (right - left) / 2;
                                if value >= state_c.edges[mid].load(Ordering::Relaxed) {
                                    left = mid + 1;
                                } else {
                                    right = mid - 1;
                                }
                            }
                            // Return the bin index
                            left
                        }
                    }
                }]
                .fetch_add(1.0 - decay_weight, Ordering::Relaxed);
            }
        });

        Self {
            dispatcher_handle,
            state,
            range,
            scaling,
        }
        .build(cx, |_| {})
    }

    fn update(&self) {
        let size: usize = self.state.size.load(Ordering::Relaxed);

        (0..size).for_each(|x| {
            let scaled = self.range.0 + (x as f32 / size as f32) * (self.range.1 - self.range.0);
            let edge = self
                .scaling
                .normalized_to_value(scaled, self.range.0, self.range.1);

            self.state.edges[x].store(edge, Ordering::Relaxed);
        });

        self.state.decay_weight.store(
            Self::decay_weight(self.state.decay, self.state.sample_rate),
            Ordering::Relaxed,
        );
    }

    fn decay_weight(decay: f32, sample_rate: f32) -> f32 {
        0.25f64.powf(((decay / 1000.0) as f64 * sample_rate as f64).recip()) as f32
    }
}

impl<B: Bus<f32> + 'static> View for Histogram<B> {
    fn element(&self) -> Option<&'static str> {
        Some("histogram")
    }
    fn draw(&self, cx: &mut DrawContext, canvas: &mut Canvas) {
        let bounds = cx.bounds();

        let line_width = cx.scale_factor();

        let x = bounds.x;
        let y = bounds.y;
        let w = bounds.w;
        let h = bounds.h;
        let h_ceil = bounds.h.ceil() as usize;

        let mut stroke = vg::Path::new();
        let size = self.state.size.load(Ordering::Relaxed);

        let nr_bins = if h_ceil != size && h_ceil < 2048 {
            self.state.size.store(h_ceil, Ordering::Relaxed);
            self.update();
            h_ceil
        } else {
            size
        };

        let largest = self
            .state
            .data
            .iter()
            .take(nr_bins)
            .skip(1)
            .map(|x| x.load(Ordering::Relaxed))
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or_default();

        stroke.move_to(
            x + self.state.data[nr_bins - 1].load(Ordering::Relaxed) * w,
            y,
        );

        if largest > 0.0 {
            for i in 0..nr_bins {
                stroke.line_to(
                    x + (self.state.data[nr_bins - i].load(Ordering::Relaxed) / largest) * w,
                    y + h * i as f32 / (nr_bins - 1) as f32,
                );
            }
        }

        let mut fill = stroke.clone();
        fill.line_to(x, y + h);
        fill.line_to(x, y);
        fill.close();
        canvas.fill_path(&fill, &vg::Paint::color(cx.background_color().into()));

        canvas.stroke_path(
            &stroke,
            &vg::Paint::color(cx.font_color().into()).with_line_width(line_width),
        );
    }
}
