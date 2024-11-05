use cyma::prelude::*;
use cyma::utils::{MonoOutlet, Outlet, OutletConsumer, PeakBuffer};
use nih_plug::editor::Editor;
use nih_plug_vizia::{assets, create_vizia_editor, vizia::prelude::*, ViziaState, ViziaTheming};
use std::sync::{Arc, Mutex};

#[derive(Lens, Clone)]
pub(crate) struct Data {
    peak_buffer: Arc<Mutex<PeakBuffer>>,
}

impl Data {
    pub(crate) fn new(outlet: MonoOutlet) -> Self {
        Self {
            peak_buffer: Arc::new(Mutex::new(PeakBuffer::new(
                outlet.get_sample_rate(),
                outlet.get_consumer(),
                10.0,
                50.0,
            ))),
        }
    }
}

impl Model for Data {}

pub(crate) fn default_state() -> Arc<ViziaState> {
    ViziaState::new(|| (800, 500))
}

pub(crate) fn create(editor_data: Data, editor_state: Arc<ViziaState>) -> Option<Box<dyn Editor>> {
    create_vizia_editor(editor_state, ViziaTheming::default(), move |cx, _| {
        assets::register_noto_sans_light(cx);
        editor_data.clone().build(cx);
        HStack::new(cx, |cx| {
            Graph::new(cx, Data::peak_buffer, (-32.0, 8.0), ValueScaling::Decibels)
                .color(Color::rgba(255, 255, 255, 160))
                .background_color(Color::rgba(255, 255, 255, 60));
            Meter::new(
                cx,
                Data::peak_buffer,
                (-32.0, 8.0),
                ValueScaling::Decibels,
                Orientation::Vertical,
            )
            .color(Color::rgba(255, 255, 255, 160))
            .background_color(Color::rgba(255, 255, 255, 60))
            .width(Pixels(24.0));
        })
        .col_between(Pixels(8.))
        .background_color(Color::rgb(16, 16, 16));
    })
}
