use cyma::prelude::*;
use cyma::utils::{MonoInlet, MonoOutlet};
use cyma::{
    utils::PeakBuffer,
    visualizers::{Graph, Grid, UnitRuler},
};
use nih_plug::editor::Editor;
use nih_plug_vizia::{assets, create_vizia_editor, vizia::prelude::*, ViziaState, ViziaTheming};
use std::sync::{Arc, Mutex};

#[derive(Lens, Clone)]
pub(crate) struct Data {}

impl Data {
    pub(crate) fn new() -> Self {
        Self {}
    }
}

impl Model for Data {}

pub(crate) fn default_state() -> Arc<ViziaState> {
    ViziaState::new(|| (800, 500))
}

pub(crate) fn create(
    editor_data: Data,
    editor_state: Arc<ViziaState>,
    outlet: MonoOutlet,
) -> Option<Box<dyn Editor>> {
    create_vizia_editor(editor_state, ViziaTheming::default(), move |cx, _| {
        assets::register_noto_sans_light(cx);
        editor_data.clone().build(cx);
        HStack::new(cx, |cx| {
            TestGraph::new(cx, outlet.clone())
                .color(Color::rgba(255, 255, 255, 160))
                .background_color(Color::rgba(255, 255, 255, 60));
        })
        .col_between(Pixels(8.))
        .background_color(Color::rgb(16, 16, 16));
    })
}
