use nih_plug_vizia::vizia::prelude::*;
use nih_plug_vizia::vizia::vg;
use std::sync::{Arc, Mutex};

use crate::utils::SpectrumOutput;

pub struct SpectrumAnalyzer {
    spectrum: Arc<Mutex<SpectrumOutput>>,
    variant: SpectrumAnalyzerVariant,
}

pub enum SpectrumAnalyzerVariant {
    BAR,
    LINE,
}

// TODO: Allow custom clamping and scaling behavior
// TODO: Allow freq / lin etc scaling
// TODO: Allow tilting by custom dB amount (c.f. FF Pro-Q)
impl SpectrumAnalyzer {
    pub fn new<LSpectrum>(
        cx: &mut Context,
        spectrum: LSpectrum,
        variant: SpectrumAnalyzerVariant,
    ) -> Handle<Self>
    where
        LSpectrum: Lens<Target = Arc<Mutex<SpectrumOutput>>>,
    {
        Self {
            spectrum: spectrum.get(cx),
            variant,
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

        let foreground =
            vg::Paint::color(cx.font_color().into()).with_line_width(cx.scale_factor());
        let background =
            vg::Paint::color(cx.background_color().into()).with_line_width(cx.scale_factor());

        match &self.variant {
            SpectrumAnalyzerVariant::BAR => {
                let mut path = vg::Path::new();

                for (bin_idx, magnitude) in spectrum.iter().enumerate() {
                    let bin_x = bin_idx as f32 / spectrum.len() as f32;

                    let magnitude_db = nih_plug::util::gain_to_db(*magnitude);
                    let height = ((magnitude_db + 80.0) / 100.0).clamp(0.0, 1.0);

                    path.move_to(x + (w * bin_x), y + (h * (1.0 - height)));
                    path.line_to(x + (w * bin_x), y + h);
                }

                canvas.stroke_path(&path, &foreground);
            }
            SpectrumAnalyzerVariant::LINE => {
                let mut magnitude_db = nih_plug::util::gain_to_db(spectrum[0]);
                let mut height = ((magnitude_db + 80.0) / 100.0).clamp(0.0, 1.0);

                let mut line = vg::Path::new();
                line.move_to(x, y + (h * (1.0 - height)));

                for (bin_idx, magnitude) in spectrum.iter().enumerate() {
                    let bin_x = bin_idx as f32 / spectrum.len() as f32;

                    magnitude_db = nih_plug::util::gain_to_db(*magnitude);
                    height = ((magnitude_db + 80.0) / 100.0).clamp(0.0, 1.0);

                    line.line_to(x + (w * bin_x), y + (h * (1.0 - height)));
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
