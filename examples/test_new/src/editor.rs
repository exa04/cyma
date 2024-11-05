use cyma::prelude::*;
use cyma::utils::{
    MonoMultiOutlet, MonoOutlet, Outlet, OutletConsumer, PeakBuffer, WaveformBuffer,
};
use nih_plug::editor::Editor;
use nih_plug_vizia::widgets::ResizeHandle;
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
                outlet.get_consumer(),
                10.0,
                50.0,
            ))),
        }
    }
}

impl Model for Data {}

pub(crate) fn default_state() -> Arc<ViziaState> {
    ViziaState::new(|| (800, 800))
}

pub(crate) fn create(editor_data: Data, editor_state: Arc<ViziaState>) -> Option<Box<dyn Editor>> {
    create_vizia_editor(editor_state, ViziaTheming::default(), move |cx, _| {
        assets::register_noto_sans_light(cx);
        editor_data.clone().build(cx);
        VStack::new(cx, |cx| {
            peak_graph(cx);
            Label::new(
                cx,
                format!("Cyma {} - Visualizers Example", env!("CARGO_PKG_VERSION")).as_str(),
            )
            .color(Color::rgb(180, 180, 180));
        })
        .background_color(Color::rgb(8, 8, 8))
        .child_space(Pixels(8.0))
        .row_between(Pixels(4.0));

        ResizeHandle::new(cx);
    })
}

fn peak_graph(cx: &mut Context) {
    HStack::new(cx, |cx| {
        ZStack::new(cx, |cx| {
            Grid::new(
                cx,
                ValueScaling::Linear,
                (-32., 8.),
                vec![6.0, 0.0, -6.0, -12.0, -18.0, -24.0, -30.0],
                Orientation::Horizontal,
            )
            .border_width(Pixels(0.5))
            .color(Color::rgb(30, 30, 30));

            Graph::new(cx, Data::peak_buffer, (-32.0, 8.0), ValueScaling::Decibels)
                .color(Color::rgba(255, 255, 255, 60))
                .background_color(Color::rgba(255, 255, 255, 30));

            UnitRuler::new(
                cx,
                (-32.0, 8.0),
                ValueScaling::Linear,
                vec![
                    (6.0, "6 dB"),
                    (0.0, "0 dB"),
                    (-6.0, "-6 dB"),
                    (-12.0, "-12 dB"),
                    (-18.0, "-18 dB"),
                    (-24.0, "-24 dB"),
                    (-30.0, "-30 dB"),
                ],
                Orientation::Vertical,
            )
            .font_size(12.)
            .color(Color::rgb(220, 220, 220))
            .right(Pixels(8.0))
            .left(Stretch(1.0));
        });

        ZStack::new(cx, |cx| {
            Meter::new(
                cx,
                Data::peak_buffer,
                (-32.0, 8.0),
                ValueScaling::Decibels,
                Orientation::Vertical,
            )
            .color(Color::rgba(255, 255, 255, 100))
            .background_color(Color::rgba(255, 255, 255, 32));
        })
        .background_color(Color::rgb(8, 8, 8))
        .width(Pixels(24.));
    })
    .background_color(Color::rgb(16, 16, 16))
    .border_width(Pixels(1.0))
    .border_color(Color::rgb(48, 48, 48));
}
