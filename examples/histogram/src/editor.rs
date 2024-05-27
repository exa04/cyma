use cyma::prelude::*;
use cyma::{
    utils::HistogramBuffer,
    visualizers::{Histogram, Grid, UnitRuler},
};
use nih_plug::editor::Editor;
use nih_plug_vizia::{assets, create_vizia_editor, vizia::prelude::*, ViziaState, ViziaTheming};
use std::sync::{Arc, Mutex};

#[derive(Lens, Clone)]
pub(crate) struct Data {
    histogram_buffer: Arc<Mutex<HistogramBuffer>>,
}

impl Data {
    pub(crate) fn new(histogram_buffer: Arc<Mutex<HistogramBuffer>>) -> Self {
        Self { histogram_buffer }
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
            ZStack::new(cx, |cx| {
                Grid::new(
                    cx,
                    ValueScaling::Linear,
                    (-32., 8.),
                    vec![6.0, 0.0, -6.0, -12.0, -18.0, -24.0, -30.0],
                    Orientation::Horizontal,
                )
                .color(Color::rgb(60, 60, 60));

                Histogram::new(cx, Data::histogram_buffer, (-32.0, 8.0))
                    .color(Color::rgba(255, 255, 255, 160))
                    .background_color(Color::rgba(255, 255, 255, 60));
            })
            .background_color(Color::rgb(16, 16, 16));

            UnitRuler::new(
                cx,
                (-32.0, 8.0),
                ValueScaling::Linear,
                vec![
                    (6.0, "6db"),
                    (0.0, "0db"),
                    (-6.0, "-6db"),
                    (-12.0, "-12db"),
                    (-18.0, "-18db"),
                    (-24.0, "-24db"),
                    (-30.0, "-30db"),
                ],
                Orientation::Vertical,
            )
            .font_size(12.)
            .color(Color::rgb(160, 160, 160))
            .width(Pixels(48.));
        })
        .col_between(Pixels(8.))
        .background_color(Color::rgb(0, 0, 0));
    })
}
