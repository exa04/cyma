use cyma::prelude::*;
use cyma::utils::{
    HistogramBuffer, PeakBuffer, RingBuffer, SpectrumInput, SpectrumOutput, WaveformBuffer,
};
use nih_plug::prelude::*;
use nih_plug_vizia::ViziaState;
use std::{
    f32::consts::PI,
    sync::{Arc, Mutex},
};

mod editor;

pub struct VisualizersDemo {
    params: Arc<DemoParams>,

    // These buffers will hold the sample data for the visualizers.
    oscilloscope_buffer: Arc<Mutex<WaveformBuffer>>,
    peak_buffer: Arc<Mutex<PeakBuffer>>,
    lissajous_buffer: Arc<Mutex<RingBuffer<(f32, f32)>>>,
    histogram_buffer: Arc<Mutex<HistogramBuffer>>,

    spectrum_input: SpectrumInput,
    spectrum_output: Arc<Mutex<SpectrumOutput>>,

    waveform: Arc<Mutex<Vec<f32>>>,
}

#[derive(Params)]
struct DemoParams {
    #[persist = "editor-state"]
    editor_state: Arc<ViziaState>,
}

impl Default for VisualizersDemo {
    fn default() -> Self {
        let (spectrum_input, spectrum_output) = SpectrumInput::new(2, 100.);

        Self {
            params: Arc::new(DemoParams::default()),
            oscilloscope_buffer: Arc::new(Mutex::new(WaveformBuffer::new(800, 5.0))),
            peak_buffer: Arc::new(Mutex::new(PeakBuffer::new(800, 10.0, 50.))),
            histogram_buffer: Arc::new(Mutex::new(HistogramBuffer::new(512, 1.0))),
            lissajous_buffer: Arc::new(Mutex::new(RingBuffer::new(2048))),

            spectrum_input,
            spectrum_output: Arc::new(Mutex::new(spectrum_output)),

            // This is just some dummy data that doesn't change.
            waveform: Arc::new(Mutex::new(
                (0..256)
                    .map(|x| {
                        let x = 2. * PI * x as f32 / 256.;
                        0.6 * (x).sin() + 0.3 * (x * 2.).sin() + 0.1 * (x * 4.).sin()
                    })
                    .collect::<Vec<f32>>(),
            )),
        }
    }
}

impl Default for DemoParams {
    fn default() -> Self {
        Self {
            editor_state: editor::default_state(),
        }
    }
}

impl Plugin for VisualizersDemo {
    const NAME: &'static str = "CymaVisualizers";
    const VENDOR: &'static str = "223230";
    const URL: &'static str = env!("CARGO_PKG_HOMEPAGE");
    const EMAIL: &'static str = "223230@pm.me";
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[AudioIOLayout {
        main_input_channels: NonZeroU32::new(2),
        main_output_channels: NonZeroU32::new(2),

        aux_input_ports: &[],
        aux_output_ports: &[],

        names: PortNames::const_default(),
    }];

    const MIDI_INPUT: MidiConfig = MidiConfig::None;
    const MIDI_OUTPUT: MidiConfig = MidiConfig::None;

    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        editor::create(
            editor::Data::new(
                self.oscilloscope_buffer.clone(),
                self.peak_buffer.clone(),
                self.histogram_buffer.clone(),
                self.lissajous_buffer.clone(),
                self.spectrum_output.clone(),
                self.waveform.clone(),
            ),
            self.params.editor_state.clone(),
        )
    }

    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        match self.oscilloscope_buffer.lock() {
            Ok(mut buffer) => {
                buffer.set_sample_rate(buffer_config.sample_rate);
            }
            Err(_) => return false,
        }
        match self.peak_buffer.lock() {
            Ok(mut buffer) => {
                buffer.set_sample_rate(buffer_config.sample_rate);
            }
            Err(_) => return false,
        }
        match self.histogram_buffer.lock() {
            Ok(mut buffer) => {
                buffer.set_sample_rate(buffer_config.sample_rate);
            }
            Err(_) => return false,
        }

        self.spectrum_input
            .update_sample_rate(buffer_config.sample_rate);

        true
    }

    fn process(
        &mut self,
        buffer: &mut nih_plug::buffer::Buffer,
        _: &mut AuxiliaryBuffers,
        _: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        // Append to the visualizers' respective buffers, only if the editor is currently open.
        if self.params.editor_state.is_open() {
            self.oscilloscope_buffer
                .lock()
                .unwrap()
                .enqueue_buffer(buffer, None);
            self.peak_buffer
                .lock()
                .unwrap()
                .enqueue_buffer(buffer, None);
            self.histogram_buffer
                .lock()
                .unwrap()
                .enqueue_buffer(buffer, None);

            if buffer.channels() > 1 {
                for mut sample in buffer.iter_samples() {
                    self.lissajous_buffer
                        .lock()
                        .unwrap()
                        .enqueue((*unsafe { sample.get_unchecked_mut(0) }, *unsafe {
                            sample.get_unchecked_mut(1)
                        }));
                }
            }

            self.spectrum_input.compute(buffer);
        }
        ProcessStatus::Normal
    }
}

impl ClapPlugin for VisualizersDemo {
    const CLAP_ID: &'static str = "org.cyma.visualizers";
    const CLAP_DESCRIPTION: Option<&'static str> =
        Some("An example plug-in showcasing all Cyma visualizers");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;

    const CLAP_FEATURES: &'static [ClapFeature] = &[ClapFeature::AudioEffect, ClapFeature::Stereo];
}

impl Vst3Plugin for VisualizersDemo {
    const VST3_CLASS_ID: [u8; 16] = *b"CYMA0VISUALIZERS";

    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Fx, Vst3SubCategory::Dynamics];
}

nih_export_clap!(VisualizersDemo);
nih_export_vst3!(VisualizersDemo);
