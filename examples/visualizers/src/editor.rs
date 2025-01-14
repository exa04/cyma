use cyma::prelude::*;
use nih_plug::editor::Editor;
use nih_plug_vizia::widgets::ResizeHandle;
use nih_plug_vizia::{assets, create_vizia_editor, vizia::prelude::*, ViziaState, ViziaTheming};
use std::sync::Arc;

pub(crate) fn default_state() -> Arc<ViziaState> {
    ViziaState::new(|| (1200, 800))
}

pub(crate) fn create(
    bus: Arc<MonoBus>,
    stereo_bus: Arc<StereoBus>,
    editor_state: Arc<ViziaState>,
) -> Option<Box<dyn Editor>> {
    create_vizia_editor(editor_state, ViziaTheming::default(), move |cx, _| {
        bus.subscribe(cx);
        stereo_bus.subscribe(cx);

        assets::register_noto_sans_light(cx);
        VStack::new(cx, |cx| {
            HStack::new(cx, |cx| {
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
                        bus.clone(),
                        10.0,
                        50.0,
                        (-32.0, 8.0),
                        ValueScaling::Decibels,
                    )
                    .color(Color::rgba(255, 255, 255, 60))
                    .background_color(Color::rgba(255, 255, 255, 30));
                    Graph::rms(
                        cx,
                        bus.clone(),
                        10.0,
                        250.0,
                        (-32.0, 8.0),
                        ValueScaling::Decibels,
                    )
                    .color(Color::rgba(255, 92, 92, 128));
                    Histogram::new(cx, bus.clone(), 250.0, (-32.0, 8.0), ValueScaling::Decibels)
                        .width(Pixels(64.0))
                        .color(Color::rgba(64, 128, 255, 64))
                        .background_color(Color::rgba(64, 128, 255, 32));
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
                    Meter::rms(
                        cx,
                        bus.clone(),
                        800.0,
                        (-32.0, 8.0),
                        ValueScaling::Decibels,
                        Orientation::Vertical,
                    )
                    .background_color(Color::rgba(255, 92, 92, 50));
                    Meter::peak(
                        cx,
                        bus.clone(),
                        400.0,
                        (-32.0, 8.0),
                        ValueScaling::Decibels,
                        Orientation::Vertical,
                    )
                    .background_color(Color::rgba(255, 255, 255, 30));
                    Meter::peak(
                        cx,
                        bus.clone(),
                        800.0,
                        (-32.0, 8.0),
                        ValueScaling::Decibels,
                        Orientation::Vertical,
                    )
                    .color(Color::rgba(255, 255, 255, 120));
                })
                .background_color(Color::rgb(8, 8, 8))
                .width(Pixels(24.0));
            })
            .background_color(Color::rgb(16, 16, 16))
            .border_width(Pixels(1.0))
            .border_color(Color::rgb(48, 48, 48));

            HStack::new(cx, |cx| {
                ZStack::new(cx, |cx| {
                    LissajousGrid::new(cx)
                        .background_color(Color::rgb(16, 16, 16))
                        .color(Color::rgb(48, 48, 48));
                    Lissajous::new(cx, stereo_bus.clone(), 2048)
                        .color(Color::rgba(255, 255, 255, 40));
                })
                .width(Pixels(200.0))
                .background_color(Color::rgb(16, 16, 16))
                .border_width(Pixels(1.0))
                .border_color(Color::rgb(48, 48, 48));
                VStack::new(cx, |cx| {
                    Oscilloscope::new(cx, bus.clone(), 4.0, (-1.0, 1.0), ValueScaling::Linear)
                        .color(Color::rgba(255, 255, 255, 120));
                })
                .background_color(Color::rgb(16, 16, 16))
                .border_width(Pixels(1.0))
                .border_color(Color::rgb(48, 48, 48));
            })
            .col_between(Pixels(4.0))
            .height(Pixels(200.0));

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
