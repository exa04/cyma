use cyma::prelude::*;
use nih_plug::editor::Editor;
use nih_plug_vizia::widgets::ResizeHandle;
use nih_plug_vizia::{assets, create_vizia_editor, vizia::prelude::*, ViziaState, ViziaTheming};
use std::sync::{Arc, Mutex};

#[derive(Lens, Clone)]
pub(crate) struct Data {
    pub(crate) spectrum: Arc<Mutex<SpectrumOutput>>,
    pub(crate) duration: f32,
}

enum EditorEvent {
    UpdateDuration(f32),
}

impl Model for Data {
    fn event(&mut self, _cx: &mut EventContext<'_>, event: &mut Event) {
        event.map(|editor_event, meta| match editor_event {
            EditorEvent::UpdateDuration(duration) => self.duration = *duration,
        });
    }
}

pub(crate) fn default_state() -> Arc<ViziaState> {
    ViziaState::new(|| (1200, 800))
}

pub(crate) fn create(
    bus: Arc<MonoBus>,
    editor_data: Data,
    stereo_bus: Arc<StereoBus>,
    editor_state: Arc<ViziaState>,
) -> Option<Box<dyn Editor>> {
    create_vizia_editor(editor_state, ViziaTheming::default(), move |cx, _| {
        bus.subscribe(cx);
        stereo_bus.subscribe(cx);

        editor_data.clone().build(cx);

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
                        Data::duration,
                        50.0,
                        (-32.0, 8.0),
                        ValueScaling::Decibels,
                    )
                    .color(Color::rgba(255, 255, 255, 60))
                    .background_color(Color::rgba(255, 255, 255, 30));
                    Graph::rms(
                        cx,
                        bus.clone(),
                        Data::duration,
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
                    Oscilloscope::new(cx, bus.clone(), 0.25, (-1.0, 1.0), ValueScaling::Linear)
                        .color(Color::rgba(255, 255, 255, 120));
                })
                .background_color(Color::rgb(16, 16, 16))
                .border_width(Pixels(1.0))
                .border_color(Color::rgb(48, 48, 48));
            })
            .col_between(Pixels(4.0))
            .height(Pixels(200.0));

            ZStack::new(cx, |cx| {
                Grid::new(
                    cx,
                    ValueScaling::Frequency,
                    (10., 21_000.),
                    vec![
                        20., 40., 30., 50., 60., 70., 80., 90., 100., 200., 300., 400., 500., 600.,
                        700., 800., 900., 1_000., 2_000., 3_000., 4_000., 5_000., 6_000., 7_000.,
                        8_000., 9_000., 10_000., 20_000.,
                    ],
                    Orientation::Vertical,
                )
                .border_width(Pixels(0.5))
                .color(Color::rgb(30, 30, 30));
                Grid::new(
                    cx,
                    ValueScaling::Linear,
                    (-80., 6.),
                    vec![0., -10., -20., -30., -40., -50., -60., -70.],
                    Orientation::Horizontal,
                )
                .border_width(Pixels(0.5))
                .color(Color::rgb(30, 30, 30));
                SpectrumAnalyzer::new(
                    cx,
                    Data::spectrum,
                    SpectrumAnalyzerVariant::LINE,
                    ValueScaling::Frequency,
                    (10., 21_000.),
                    ValueScaling::Decibels,
                    (-110., 6.),
                )
                .with_slope(4.5)
                .color(Color::rgba(255, 255, 255, 60))
                .background_color(Color::rgba(255, 255, 255, 30));
                Element::new(cx)
                    .background_gradient(
                        LinearGradientBuilder::with_direction("to bottom")
                            .add_stop(Color::transparent())
                            .add_stop(Color::rgb(16, 16, 16)),
                    )
                    .height(Pixels(48.))
                    .top(Stretch(1.));
                UnitRuler::new(
                    cx,
                    (10., 21_000.),
                    ValueScaling::Frequency,
                    vec![
                        (20., "20"),
                        (50., "50"),
                        (100., "100"),
                        (200., "200"),
                        (500., "500"),
                        (1_000., "1k"),
                        (2_000., "2k"),
                        (5_000., "5k"),
                        (10_000., "10k"),
                    ],
                    Orientation::Horizontal,
                )
                .height(Pixels(16.))
                .font_size(12.)
                .color(Color::rgb(160, 160, 160))
                .top(Stretch(1.))
                .bottom(Pixels(4.));
            })
            .background_color(Color::rgb(16, 16, 16))
            .border_width(Pixels(1.0))
            .border_color(Color::rgb(48, 48, 48))
            .height(Pixels(200.0));

            HStack::new(cx, |cx| {
                Label::new(
                    cx,
                    format!("Cyma {} - Visualizers Example", env!("CARGO_PKG_VERSION")).as_str(),
                )
                .color(Color::rgb(180, 180, 180))
                .right(Pixels(16.0));

                Label::new(cx, "Duration")
                    .color(Color::rgb(180, 180, 180))
                    .right(Pixels(4.0));
                Slider::new(cx, Data::duration)
                    .range(2.0..16.0)
                    .step(2.0)
                    .on_changing(|cx, value| cx.emit(EditorEvent::UpdateDuration(value)))
                    .width(Pixels(120.0))
                    .right(Pixels(4.0));
                Label::new(cx, Data::duration.map(|d| format!("{:.2}s", d)))
                    .color(Color::rgb(240, 240, 240));
            })
            .child_top(Stretch(1.))
            .child_bottom(Stretch(1.))
            .height(Pixels(16.0));
        })
        .background_color(Color::rgb(8, 8, 8))
        .child_space(Pixels(8.0))
        .row_between(Pixels(4.0));

        ResizeHandle::new(cx);
    })
}
