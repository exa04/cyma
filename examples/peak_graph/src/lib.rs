use cyma::prelude::*;
use nih_plug::prelude::*;
use nih_plug_vizia::ViziaState;
use std::sync::{Arc, Mutex};

mod editor;

pub struct PeakGraphPlugin {
    params: Arc<DemoParams>,
    bus: Arc<MonoBus>,
}

#[derive(Params)]
struct DemoParams {
    #[persist = "editor-state"]
    editor_state: Arc<ViziaState>,
}

impl Default for PeakGraphPlugin {
    fn default() -> Self {
        Self {
            params: Arc::new(DemoParams::default()),
            bus: Arc::new(Default::default()),
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

impl Plugin for PeakGraphPlugin {
    const NAME: &'static str = "CymaPeakGraph";
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
        editor::create(self.params.editor_state.clone(), self.bus.clone())
    }

    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        self.bus.set_sample_rate(buffer_config.sample_rate);

        true
    }

    fn process(
        &mut self,
        buffer: &mut nih_plug::buffer::Buffer,
        _: &mut AuxiliaryBuffers,
        _: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        // Push samples into the bus, only if the editor is currently open.
        if self.params.editor_state.is_open() {
            self.bus.send_buffer_summing(buffer);
        }
        ProcessStatus::Normal
    }
}

impl ClapPlugin for PeakGraphPlugin {
    const CLAP_ID: &'static str = "org.cyma.peak_graph";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("A peak graph built using Cyma");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;

    const CLAP_FEATURES: &'static [ClapFeature] =
        &[ClapFeature::AudioEffect, ClapFeature::Analyzer];
}

impl Vst3Plugin for PeakGraphPlugin {
    const VST3_CLASS_ID: [u8; 16] = *b"CYMA000PEAKGRAPH";

    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] = &[Vst3SubCategory::Analyzer];
}

nih_export_clap!(PeakGraphPlugin);
nih_export_vst3!(PeakGraphPlugin);
