use cyma::prelude::*;
use cyma::visualizers::{Graph, Grid, UnitRuler};
use nih_plug::editor::Editor;
use nih_plug::nih_dbg;
use std::sync::Arc;
use vizia_plug::{create_vizia_editor, vizia::prelude::*, ViziaState, ViziaTheming};

pub(crate) fn default_state() -> Arc<ViziaState> {
    ViziaState::new(|| (800, 500))
}

#[derive(Lens)]
struct TimerState {
    timer: Timer,
}

impl Model for TimerState {}

pub(crate) fn create(editor_state: Arc<ViziaState>, bus: Arc<MonoBus>) -> Option<Box<dyn Editor>> {
    create_vizia_editor(editor_state, ViziaTheming::default(), move |cx, _| {
        bus.subscribe(cx);

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

                Graph::peak(
                    cx,
                    bus.clone(),
                    10.0,
                    50.0,
                    (-32.0, 8.0),
                    ValueScaling::Decibels,
                )
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
        .gap(Pixels(8.))
        .background_color(Color::rgb(0, 0, 0));
    })
}
