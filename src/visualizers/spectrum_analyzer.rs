use nih_plug_vizia::vizia::prelude::*;
use nih_plug_vizia::vizia::vg;
use std::sync::{Arc, Mutex};

use crate::utils::SpectrumOutput;
use crate::utils::ValueScaling;

/// A spectrum analyzer that shows the magnitude of each frequency inside a
/// [`SpectrumOutput`].
///
/// Can either display magnitude as discrete bars, or as a graph.
///
/// # Usage
///
/// To use this visualizer, you need to add a
/// [`SpectrumInput`](crate::utils::SpectrumInput) and a
/// [`SpectrumOutput`](crate::utils::SpectrumOutput) to your plugin. Call
/// [`compute`](crate::utils::SpectrumInput::compute) on the `SpectrumInput` to
/// compute the spectrum inside your plug-in's
/// [`process()`](nih_plug::plugin::Plugin::process) function. The `SpectrumOutput`
/// can then be sent to your editor inside the
/// [`editor()`](nih_plug::plugin::Plugin::editor) function.
///
/// Here's a detailed guide on how to do this.
///
/// First, add the spectrum input and output as fields of your
/// [`Plugin`](nih_plug::plugin::Plugin).
///
/// ```
/// pub struct MyPlugin {
///     spectrum_input: SpectrumInput,
///     spectrum_output: Arc<Mutex<SpectrumOutput>>,
/// }
/// ```
///
/// In your `default()` function, you can now create them like so:
///
/// ```
/// impl Default for MyPlugin {
///     fn default() -> Self {
///         let (spectrum_input, spectrum_output) = SpectrumInput::new(2, 100.);
///         Self {
///             spectrum_input,
///             spectrum_output: Arc::new(Mutex::new(spectrum_output))
///         }
///     }
/// }
/// ```
///
/// In order to correctly determine frequencies, the `SpectrumInput` also needs to
/// know the plug-in host sample rate. Update it with the correct sample rate inside
/// your [`initialize()`](nih_plug::plugin::Plugin::initialize) function.
///
/// ```
/// fn initialize(
///     &mut self,
///     _audio_io_layout: &AudioIOLayout,
///     buffer_config: &BufferConfig,
///     _context: &mut impl InitContext<Self>,
/// ) -> bool {
///     self.spectrum_input
///         .update_sample_rate(buffer_config.sample_rate);
///     true
/// }
/// ```
///
/// Now, you can compute the spectrum for each buffer passed to your
/// [`process()`](nih_plug::plugin::Plugin::process) function.
///
/// ```
/// fn process(
///     &mut self,
///     buffer: &mut nih_plug::buffer::Buffer,
///     _: &mut AuxiliaryBuffers,
///     _: &mut impl ProcessContext<Self>,
/// ) -> ProcessStatus {
///     if self.params.editor_state.is_open() {
///         self.spectrum_input.compute(buffer);
///     }
/// }
/// ```
///
/// To display the spectrum, you will now need to pass it to your editor. First, add
/// the appropriate field to its `Data` struct.
///
/// ```
/// #[derive(Lens, Clone)]
/// pub(crate) struct Data {
///     pub(crate) spectrum: Arc<Mutex<SpectrumOutput>>,
/// }
///
/// impl Data {
///     pub(crate) fn new(
///         spectrum: Arc<Mutex<SpectrumOutput>>,
///     ) -> Self {
///         Self {
///             spectrum,
///         }
///     }
/// }
/// ```
///
/// Now, upon creation, you can clone a reference to the `Arc<Mutex>>` and send it
/// off to the editor.
///
/// ```
/// fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
///     editor::create(
///         editor::Data::new(
///             self.spectrum_output.clone()
///         ),
///         self.params.editor_state.clone()
///     )
/// }
/// ```
///
/// Finally, you can now add the `SpectrumAnalyzer` to your editor!
///
/// ```
/// pub(crate) fn create(editor_data: Data, editor_state: Arc<ViziaState>) -> Option<Box<dyn Editor>> {
///     create_vizia_editor(editor_state, ViziaTheming::default(), move |cx, _| {
///         assets::register_noto_sans_light(cx);
///         editor_data.clone().build(cx);
///         ZStack::new(cx, |cx| {
///             SpectrumAnalyzer::new(
///                 cx,
///                 Data::spectrum,
///                 SpectrumAnalyzerVariant::LINE,
///                 ValueScaling::Frequency,
///                 (10., 21_000.),
///                 ValueScaling::Decibels,
///                 (-110., 6.),
///             )
///             .color(Color::rgba(255, 255, 255, 160))
///             .background_color(Color::rgba(255, 255, 255, 60));
///         })
///         .background_color(Color::rgb(16, 16, 16));
///     })
/// }
/// ```
///
/// # Example
///
/// Here's how to compose a spectrum analyzer with a slope applied to it, a grid
/// backdrop, and a unit ruler.
///
/// ```
/// ZStack::new(cx, |cx| {
///     Grid::new(
///         cx,
///         ValueScaling::Frequency,
///         (10., 21_000.),
///         vec![
///             20., 40., 30., 50., 60., 70., 80., 90., 100., 200., 300., 400., 500., 600., 700.,
///             800., 900., 1_000., 2_000., 3_000., 4_000., 5_000., 6_000., 7_000., 8_000., 9_000.,
///             10_000., 20_000.,
///         ],
///         Orientation::Vertical,
///     )
///     .color(Color::rgb(60, 60, 60));
///     Grid::new(
///         cx,
///         ValueScaling::Linear,
///         (-80., 6.),
///         vec![0., -10., -20., -30., -40., -50., -60., -70.],
///         Orientation::Horizontal,
///     )
///     .color(Color::rgb(40, 40, 40));
///     SpectrumAnalyzer::new(
///         cx,
///         Data::spectrum,
///         SpectrumAnalyzerVariant::LINE,
///         ValueScaling::Frequency,
///         (10., 21_000.),
///         ValueScaling::Decibels,
///         (-110., 6.),
///     )
///     .with_slope(4.5)
///     .color(Color::rgba(255, 255, 255, 160))
///     .background_color(Color::rgba(255, 255, 255, 60));
///     // Displays a fade to the background color at the bottom, as a backdrop for the unit ruler
///     Element::new(cx)
///         .background_gradient(
///             LinearGradientBuilder::with_direction("to bottom")
///                 .add_stop(Color::transparent())
///                 .add_stop(Color::rgb(16, 16, 16)),
///         )
///         .height(Pixels(48.))
///         .top(Stretch(1.));
///     UnitRuler::new(
///         cx,
///         (10., 21_000.),
///         ValueScaling::Frequency,
///         vec![
///             (20., "20"),
///             (50., "50"),
///             (100., "100"),
///             (200., "200"),
///             (500., "500"),
///             (1_000., "1k"),
///             (2_000., "2k"),
///             (5_000., "5k"),
///             (10_000., "10k"),
///         ],
///         Orientation::Horizontal,
///     )
///     .height(Pixels(16.))
///     .font_size(12.)
///     .color(Color::rgb(160, 160, 160))
///     .top(Stretch(1.))
///     .bottom(Pixels(8.));
/// })
/// .background_color(Color::rgb(16, 16, 16))
/// .border_color(Color::rgb(80, 80, 80))
/// .border_width(Pixels(1.));
/// ```
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
