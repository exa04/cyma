use nih_plug_vizia::vizia::prelude::*;
use nih_plug_vizia::vizia::vg;
use std::sync::{Arc, Mutex};

use crate::utils::SpectrumOutput;
use crate::utils::ValueScaling;

pub struct SpectrumAnalyzer {
    spectrum: Arc<Mutex<SpectrumOutput>>,
    variant: SpectrumAnalyzerVariant,
    frequency_scaling: ValueScaling,
    magnitude_scaling: ValueScaling,
}

pub enum SpectrumAnalyzerVariant {
    BAR,
    LINE,
}

// TODO: Allow tilting by custom dB amount (c.f. FF Pro-Q)
impl SpectrumAnalyzer {
    pub fn new<LSpectrum>(
        cx: &mut Context,
        spectrum: LSpectrum,
        variant: SpectrumAnalyzerVariant,
        frequency_scaling: ValueScaling,
        magnitude_scaling: ValueScaling,
    ) -> Handle<Self>
    where
        LSpectrum: Lens<Target = Arc<Mutex<SpectrumOutput>>>,
    {
        Self {
            spectrum: spectrum.get(cx),
            variant,
            frequency_scaling,
            magnitude_scaling,
        }
        .build(cx, |_cx| ())
    }
}

impl View for SpectrumAnalyzer {
    fn element(&self) -> Option<&'static str> {
        Some("spectrum-analyzer")
    }

    fn draw(&self, cx: &mut DrawContext, canvas: &mut Canvas) {
        let bounds = cx.bounds();

        let x = bounds.x;
        let y = bounds.y;
        let w = bounds.w;
        let h = bounds.h;

        let mut spectrum = self.spectrum.lock().unwrap();
        let spectrum = spectrum.read();

        let half_nyquist = 20_000.0;

        let foreground =
            vg::Paint::color(cx.font_color().into()).with_line_width(cx.scale_factor());
        let background =
            vg::Paint::color(cx.background_color().into()).with_line_width(cx.scale_factor());

        match &self.variant {
            SpectrumAnalyzerVariant::BAR => {
                let mut path = vg::Path::new();

                for (bin_idx, magnitude) in spectrum.iter().enumerate() {
                    let freq = (bin_idx as f32 / spectrum.len() as f32) * half_nyquist;

                    // Normalize frequency
                    let freq_normalized =
                        self.frequency_scaling
                            .value_to_normalized(freq, 20., half_nyquist);

                    let magnitude_normalized = self
                        .magnitude_scaling
                        .value_to_normalized(*magnitude, -80., 6.);

                    path.move_to(
                        x + (w * freq_normalized),
                        y + (h * (1.0 - magnitude_normalized)),
                    );
                    path.line_to(x + (w * freq_normalized), y + h);
                }

                canvas.stroke_path(&path, &foreground);
            }
            SpectrumAnalyzerVariant::LINE => {
                let magnitude_normalized =
                    self.magnitude_scaling
                        .value_to_normalized(spectrum[0], -80., 6.);

                let mut line = vg::Path::new();
                line.move_to(x, y + (h * (1.0 - magnitude_normalized)));

                for (bin_idx, magnitude) in spectrum.iter().enumerate() {
                    let freq = (bin_idx as f32 / spectrum.len() as f32) * half_nyquist;

                    // Normalize frequency
                    let freq_normalized =
                        self.frequency_scaling
                            .value_to_normalized(freq, 20., half_nyquist);

                    let magnitude_normalized = self
                        .magnitude_scaling
                        .value_to_normalized(*magnitude, -80., 6.);

                    line.line_to(
                        x + (w * freq_normalized),
                        y + (h * (1.0 - magnitude_normalized)),
                    );
                }

                let mut fill = line.clone();
                fill.line_to(x + w, y + h);
                fill.line_to(x, y + h);

                fill.close();

                canvas.fill_path(&fill, &background);
                canvas.stroke_path(&line, &foreground);
            }
        }
    }
}
