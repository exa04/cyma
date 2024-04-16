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
    slope: Option<f32>,
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
            slope: None,
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

                // This will be used to normalize the magnitudes if a slope gets applied to them
                let magnitude_slope_divisor = if self.slope.is_some() {
                    half_nyquist.log2().powf(self.slope.unwrap()) / self.slope.unwrap()
                } else {
                    0.
                };

                for (bin_idx, magnitude) in spectrum_output.iter().enumerate() {
                    let freq = (bin_idx as f32 / spectrum_output.len() as f32) * half_nyquist;

                    // Normalize frequency
                    let freq_normalized = self.frequency_scaling.value_to_normalized(
                        freq,
                        self.frequency_range.0,
                        self.frequency_range.1,
                    );

                    // Normalize magnitude and apply slope if one is set
                    let magnitude_normalized = if self.slope.is_some() {
                        self.magnitude_scaling.value_to_normalized(
                            *magnitude
                                * ((freq + 1.).log2().powf(self.slope.unwrap())
                                    / magnitude_slope_divisor),
                            self.magnitude_range.0,
                            self.magnitude_range.1,
                        )
                    } else {
                        self.magnitude_scaling.value_to_normalized(
                            *magnitude,
                            self.magnitude_range.0,
                            self.magnitude_range.1,
                        )
                    };

                    path.move_to(
                        x + (w * freq_normalized),
                        y + (h * (1.0 - magnitude_normalized)),
                    );
                    path.line_to(x + (w * freq_normalized), y + h);
                }

                canvas.stroke_path(&path, &foreground);
            }
            SpectrumAnalyzerVariant::LINE => {
                let mut line = vg::Path::new();

                let mut magnitude_normalized = self.magnitude_scaling.value_to_normalized(
                    spectrum_output[1],
                    self.magnitude_range.0,
                    self.magnitude_range.1,
                );

                line.move_to(x, y + (h * (1.0 - magnitude_normalized)));

                // This will be used to normalize the magnitudes if a slope gets applied to them
                let magnitude_slope_divisor = if self.slope.is_some() {
                    half_nyquist.log2().powf(self.slope.unwrap()) / self.slope.unwrap()
                } else {
                    0.
                };

                for (bin_idx, magnitude) in spectrum_output.iter().skip(1).enumerate() {
                    let freq = (bin_idx as f32 / spectrum_output.len() as f32) * half_nyquist;

                    // Normalize magnitude and apply slope if one is set
                    magnitude_normalized = if self.slope.is_some() {
                        self.magnitude_scaling.value_to_normalized(
                            *magnitude
                                * ((freq + 1.).log2().powf(self.slope.unwrap())
                                    / magnitude_slope_divisor),
                            self.magnitude_range.0,
                            self.magnitude_range.1,
                        )
                    } else {
                        self.magnitude_scaling.value_to_normalized(
                            *magnitude,
                            self.magnitude_range.0,
                            self.magnitude_range.1,
                        )
                    };

                    // Skip frequencies that are out of range
                    if freq < self.frequency_range.0 {
                        line.move_to(x, y + (h * (1.0 - magnitude_normalized)));
                        continue;
                    }
                    if freq > self.frequency_range.1 {
                        break;
                    }

                    // Normalize frequency
                    let freq_normalized = self.frequency_scaling.value_to_normalized(
                        freq,
                        self.frequency_range.0,
                        self.frequency_range.1,
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

pub trait SpectrumAnalyzerModifiers {
    fn with_slope(self, slope: f32) -> Self;
}
impl SpectrumAnalyzerModifiers for Handle<'_, SpectrumAnalyzer> {
    /// Sets a slope in db/oct.
    ///
    /// Useful for spectrum analyzers that need to emphasize the highs more, in order to
    /// match a certain noise profile. For example, you can set the slope to 4.5 db/oct
    /// to approximate the spectral profile of brownian noise.
    fn with_slope(self, slope: f32) -> Self {
        self.modify(|spectrum| spectrum.slope = Some(slope))
    }
}
