use cyma::prelude::*;
use nih_plug::prelude::*;
use nih_plug_vizia::ViziaState;
use std::sync::{Arc, Mutex};

mod editor;

pub struct VisualizersPlugin {
    params: Arc<DemoParams>,
    bus: Arc<MonoBus>,
}

#[derive(Params)]
struct DemoParams {
    #[persist = "editor-state"]
    editor_state: Arc<ViziaState>,
}

impl Default for VisualizersPlugin {
    fn default() -> Self {
        Self {
            params: Arc::new(DemoParams::default()),
            bus: MonoBus::default().into(),
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

impl Plugin for VisualizersPlugin {
    const NAME: &'static str = "Cyma Visualizers Example";
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
            editor::Data::new(self.bus.clone()),
            self.params.editor_state.clone(),
        )
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
        buffer: &mut Buffer,
        _: &mut AuxiliaryBuffers,
        _: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        if self.params.editor_state.is_open() {
            self.bus.send_buffer(buffer);
        }
        ProcessStatus::Normal
    }
}

impl ClapPlugin for VisualizersPlugin {
    const CLAP_ID: &'static str = "org.cyma.visualizers";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("A lot of visualizers built using Cyma");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;

    const CLAP_FEATURES: &'static [ClapFeature] =
        &[ClapFeature::AudioEffect, ClapFeature::Analyzer];
}

impl Vst3Plugin for VisualizersPlugin {
    const VST3_CLASS_ID: [u8; 16] = *b"Cyma_Visualizers";

    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] = &[Vst3SubCategory::Analyzer];
}

nih_export_clap!(VisualizersPlugin);
nih_export_vst3!(VisualizersPlugin);
