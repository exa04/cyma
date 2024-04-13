use std::sync::{Arc, Mutex};

use cyma::{
    utils::{PeakBuffer, RingBuffer, ValueScaling, WaveformBuffer},
    visualizers::{
        Graph, Grid, Lissajous, LissajousGrid, Meter, Oscilloscope, UnitRuler, Waveform,
    },
};
use nih_plug::editor::Editor;
use nih_plug_vizia::{assets, create_vizia_editor, vizia::prelude::*, ViziaState, ViziaTheming};

#[derive(Lens, Clone)]
pub(crate) struct Data {
    pub(crate) oscilloscope_buffer: Arc<Mutex<WaveformBuffer>>,
    pub(crate) peak_buffer: Arc<Mutex<PeakBuffer>>,
    pub(crate) lissajous_buffer: Arc<Mutex<RingBuffer<(f32, f32)>>>,

    pub(crate) waveform: Arc<Mutex<Vec<f32>>>,
}

impl Data {
    pub(crate) fn new(
        oscilloscope_buffer: Arc<Mutex<WaveformBuffer>>,
        peak_buffer: Arc<Mutex<PeakBuffer>>,
        lissajous_buffer: Arc<Mutex<RingBuffer<(f32, f32)>>>,
        waveform: Arc<Mutex<Vec<f32>>>,
    ) -> Self {
        Self {
            oscilloscope_buffer,
            peak_buffer,
            lissajous_buffer,
            waveform,
        }
    }
}

impl Model for Data {}

pub(crate) fn default_state() -> Arc<ViziaState> {
    ViziaState::new(|| (800, 600))
}

pub(crate) fn create(editor_data: Data, editor_state: Arc<ViziaState>) -> Option<Box<dyn Editor>> {
    create_vizia_editor(editor_state, ViziaTheming::default(), move |cx, _| {
        assets::register_noto_sans_light(cx);
        editor_data.clone().build(cx);
        VStack::new(cx, |cx| {
            HStack::new(cx, |cx| {
                lissajous(cx);

                oscilloscope(cx);
            })
            .height(Pixels(200.))
            .col_between(Pixels(16.0));
            peak_graph(cx);
        })
        .child_space(Pixels(16.0))
        .row_between(Pixels(16.0))
        .background_color(Color::rgb(24, 24, 24));
    })
}

/// Draws a lissajous with a diamond-shaped grid backdrop and text labels for
/// positive and negative L/R signals.
fn lissajous(cx: &mut Context) {
    ZStack::new(cx, |cx| {
        LissajousGrid::new(cx)
            .background_color(Color::rgb(32, 32, 32))
            .color(Color::rgb(60, 60, 60));
        Lissajous::new(cx, Data::lissajous_buffer).color(Color::rgb(160, 160, 160));
        ZStack::new(cx, |cx| {
            Label::new(cx, "+L").color(Color::rgb(160, 160, 160));
            Label::new(cx, "+R")
                .left(Percentage(100.))
                .transform(Transform::TranslateX(LengthOrPercentage::Percentage(-100.)))
                .color(Color::rgb(160, 160, 160));
            Label::new(cx, "-R")
                .top(Percentage(100.))
                .transform(Transform::TranslateY(LengthOrPercentage::Percentage(-100.)))
                .color(Color::rgb(160, 160, 160));
            Label::new(cx, "-L")
                .top(Percentage(100.))
                .left(Percentage(100.))
                .transform(vec![
                    Transform::TranslateX(LengthOrPercentage::Percentage(-100.)),
                    Transform::TranslateY(LengthOrPercentage::Percentage(-100.)),
                ])
                .color(Color::rgb(160, 160, 160));
        })
        .space(Pixels(16.));
    })
    .background_color(Color::rgb(16, 16, 16))
    .border_color(Color::rgb(80, 80, 80))
    .border_width(Pixels(1.))
    .width(Pixels(200.));
}

/// Draws a peak graph with a grid backdrop, unit ruler, and a peak meter to side.
fn peak_graph(cx: &mut Context) {
    HStack::new(cx, |cx| {
        ZStack::new(cx, |cx| {
            Grid::new(
                cx,
                (-32.0, 8.0),
                0.0,
                vec![6.0, 0.0, -6.0, -12.0, -18.0, -24.0, -30.0],
            )
            .color(Color::rgb(60, 60, 60));

            Graph::new(cx, Data::peak_buffer, (-32.0, 6.0), ValueScaling::Decibels)
                .color(Color::rgba(255, 255, 255, 160))
                .background_color(Color::rgba(255, 255, 255, 60));
        })
        .background_color(Color::rgb(16, 16, 16));

        UnitRuler::new(
            cx,
            (-32.0, 8.0),
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
        .width(Pixels(32.));

        Meter::new(
            cx,
            Data::peak_buffer,
            (-32.0, 6.0),
            ValueScaling::Decibels,
            Orientation::Vertical,
        )
        .width(Pixels(32.0))
        .background_color(Color::rgb(60, 60, 60));
    })
    .col_between(Pixels(8.))
    .border_color(Color::rgb(80, 80, 80))
    .border_width(Pixels(1.));
}

/// Draws an oscilloscope with a grid backdrop.
fn oscilloscope(cx: &mut Context) {
    ZStack::new(cx, |cx| {
        Grid::new(cx, (-1.2, 1.2), 10.0, vec![0.0, 0.5, -0.5, 1.0, -1.0])
            .color(Color::rgb(60, 60, 60));
        Oscilloscope::new(
            cx,
            Data::oscilloscope_buffer,
            (-1.2, 1.2),
            ValueScaling::Linear,
        )
        .color(Color::rgba(0, 0, 0, 0))
        .background_color(Color::rgba(255, 255, 255, 120));
    })
    .border_color(Color::rgb(80, 80, 80))
    .border_width(Pixels(1.))
    .background_color(Color::rgb(16, 16, 16));
}
