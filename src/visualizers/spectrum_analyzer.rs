use nih_plug_vizia::vizia::prelude::*;
use nih_plug_vizia::vizia::vg;
use std::sync::{Arc, Mutex};

use crate::utils::SpectrumOutput;
use crate::utils::ValueScaling;

pub struct SpectrumAnalyzer {
    spectrum: Arc<Mutex<SpectrumOutput>>,
    variant: SpectrumAnalyzerVariant,
    frequency_scaling: ValueScaling,
    frequency_range: (f32, f32),
    magnitude_scaling: ValueScaling,
    magnitude_range: (f32, f32),
}

pub enum SpectrumAnalyzerVariant {
    BAR,
    LINE,
}

impl SpectrumAnalyzer {
    pub fn new<LSpectrum>(
        cx: &mut Context,
        spectrum: LSpectrum,
        variant: SpectrumAnalyzerVariant,
        frequency_scaling: ValueScaling,
        frequency_range: (f32, f32),
        magnitude_scaling: ValueScaling,
        magnitude_range: (f32, f32),
    ) -> Handle<Self>
    where
        LSpectrum: Lens<Target = Arc<Mutex<SpectrumOutput>>>,
    {
        Self {
            spectrum: spectrum.get(cx),
            variant,
            frequency_scaling,
            frequency_range,
            magnitude_scaling,
            magnitude_range,
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
        let half_nyquist = spectrum.sample_rate / 2.;
        let spectrum_output = spectrum.output.read();

        let foreground =
            vg::Paint::color(cx.font_color().into()).with_line_width(cx.scale_factor());
        let background =
            vg::Paint::color(cx.background_color().into()).with_line_width(cx.scale_factor());

        match &self.variant {
            SpectrumAnalyzerVariant::BAR => {
                let mut path = vg::Path::new();

                for (bin_idx, magnitude) in spectrum_output.iter().enumerate() {
                    let freq = (bin_idx as f32 / spectrum_output.len() as f32) * half_nyquist;

                    // Normalize frequency
                    let freq_normalized = self.frequency_scaling.value_to_normalized(
                        freq,
                        self.frequency_range.0,
                        self.frequency_range.1,
                    );

                    let magnitude_normalized = self.magnitude_scaling.value_to_normalized(
                        *magnitude,
                        self.magnitude_range.0,
                        self.magnitude_range.1,
                    );

                    path.move_to(
                        x + (w * freq_normalized),
                        y + (h * (1.0 - magnitude_normalized)),
                    );
                    path.line_to(x + (w * freq_normalized), y + h);
                }

                canvas.stroke_path(&path, &foreground);
            }
            SpectrumAnalyzerVariant::LINE => {
                let magnitude_normalized = self.magnitude_scaling.value_to_normalized(
                    spectrum_output[0],
                    self.magnitude_range.0,
                    self.magnitude_range.1,
                );

                let mut line = vg::Path::new();
                line.move_to(x, y + (h * (1.0 - magnitude_normalized)));

                for (bin_idx, magnitude) in spectrum_output.iter().enumerate() {
                    let freq = (bin_idx as f32 / spectrum_output.len() as f32) * half_nyquist;

                    // Normalize frequency
                    let freq_normalized = self.frequency_scaling.value_to_normalized(
                        freq,
                        self.frequency_range.0,
                        self.frequency_range.1,
                    );

                    let magnitude_normalized = self.magnitude_scaling.value_to_normalized(
                        *magnitude,
                        self.magnitude_range.0,
                        self.magnitude_range.1,
                    );

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
