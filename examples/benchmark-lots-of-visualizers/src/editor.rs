use cyma::prelude::*;
use nih_plug::editor::Editor;
use nih_plug_vizia::widgets::ResizeHandle;
use nih_plug_vizia::{assets, create_vizia_editor, vizia::prelude::*, ViziaState, ViziaTheming};
use std::sync::Arc;

#[derive(Lens, Clone)]
pub(crate) struct Data {
    bus: Arc<MonoBus>,
}

impl Data {
    pub(crate) fn new(bus: Arc<MonoBus>) -> Self {
        Self { bus }
    }
}

impl Model for Data {}

pub(crate) fn default_state() -> Arc<ViziaState> {
    ViziaState::new(|| (1920, 1080))
}

const W: usize = 16;
const H: usize = 12;

pub(crate) fn create(editor_data: Data, editor_state: Arc<ViziaState>) -> Option<Box<dyn Editor>> {
    const SPACING: Units = Pixels(1.0);

    create_vizia_editor(editor_state, ViziaTheming::default(), move |cx, _| {
        assets::register_noto_sans_light(cx);
        editor_data.clone().build(cx);
        VStack::new(cx, |cx| {
            for _ in 0..H {
                HStack::new(cx, |cx| {
                    for _ in 0..W {
                        visualizer(cx);
                    }
                })
                .row_between(SPACING);
            }

            Label::new(
                cx,
                format!("Cyma {} - Lots of Visualizers", env!("CARGO_PKG_VERSION")).as_str(),
            )
            .space(Pixels(8.0))
            .color(Color::rgb(180, 180, 180));
        })
        .background_color(Color::rgb(0, 0, 0))
        .row_between(SPACING);

        ResizeHandle::new(cx);
    })
}

fn visualizer(cx: &mut Context) {
    ZStack::new(cx, |cx| {
        Grid::new(
            cx,
            ValueScaling::Linear,
            (-32., 8.0),
            vec![6.0, 0.0, -6.0, -12.0, -18.0, -24.0, -30.0],
            Orientation::Horizontal,
        )
        .border_width(Pixels(0.5))
        .color(Color::rgb(30, 30, 30));
        Graph::peak(
            cx,
            Data::bus,
            10.0,
            50.0,
            (-32.0, 8.0),
            ValueScaling::Decibels,
        )
        .color(Color::rgba(255, 255, 255, 60))
        .background_color(Color::rgba(255, 255, 255, 30));
        Graph::rms(
            cx,
            Data::bus,
            10.0,
            250.0,
            (-32.0, 8.0),
            ValueScaling::Decibels,
        )
        .color(Color::rgba(255, 92, 92, 128));
    })
    .background_color(Color::rgb(16, 16, 16));
}
