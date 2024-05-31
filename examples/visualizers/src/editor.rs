use cyma::prelude::*;
use cyma::{
    utils::{HistogramBuffer, PeakBuffer, RingBuffer, SpectrumOutput, WaveformBuffer},
    visualizers::{
        Graph, Grid, Lissajous, LissajousGrid, Meter, Oscilloscope, SpectrumAnalyzer,
        SpectrumAnalyzerModifiers, SpectrumAnalyzerVariant, UnitRuler, Waveform,
    },
};
use nih_plug::editor::Editor;
use nih_plug_vizia::{assets, create_vizia_editor, vizia::prelude::*, ViziaState, ViziaTheming};
use std::sync::{Arc, Mutex};

#[derive(Lens, Clone)]
pub(crate) struct Data {
    pub(crate) oscilloscope_buffer: Arc<Mutex<WaveformBuffer>>,
    pub(crate) peak_buffer: Arc<Mutex<PeakBuffer>>,
    pub(crate) histogram_buffer: Arc<Mutex<HistogramBuffer>>,
    pub(crate) lissajous_buffer: Arc<Mutex<RingBuffer<(f32, f32)>>>,
    pub(crate) spectrum: Arc<Mutex<SpectrumOutput>>,

    pub(crate) waveform: Arc<Mutex<Vec<f32>>>,
}

impl Data {
    pub(crate) fn new(
        oscilloscope_buffer: Arc<Mutex<WaveformBuffer>>,
        peak_buffer: Arc<Mutex<PeakBuffer>>,
        histogram_buffer: Arc<Mutex<HistogramBuffer>>,
        lissajous_buffer: Arc<Mutex<RingBuffer<(f32, f32)>>>,
        spectrum: Arc<Mutex<SpectrumOutput>>,
        waveform: Arc<Mutex<Vec<f32>>>,
    ) -> Self {
        Self {
            oscilloscope_buffer,
            peak_buffer,
            histogram_buffer,
            lissajous_buffer,
            spectrum,
            waveform,
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
            HStack::new(cx, |cx| {
                lissajous(cx);

                oscilloscope(cx);
            })
            .height(Pixels(200.))
            .col_between(Pixels(16.0));

            peak_graph(cx);

            spectrum_analyzer(cx);
        })
        .child_space(Pixels(16.0))
        .row_between(Pixels(16.0))
        .background_color(Color::rgb(24, 24, 24));
    })
}

// Draws a spectrum analyzer with a grid backdrop and a frequency ruler.
fn spectrum_analyzer(cx: &mut Context) {
    ZStack::new(cx, |cx| {
        Grid::new(
            cx,
            ValueScaling::Frequency,
            (10., 21_000.),
            vec![
                20., 40., 30., 50., 60., 70., 80., 90., 100., 200., 300., 400., 500., 600., 700.,
                800., 900., 1_000., 2_000., 3_000., 4_000., 5_000., 6_000., 7_000., 8_000., 9_000.,
                10_000., 20_000.,
            ],
            Orientation::Vertical,
        )
        .color(Color::rgb(60, 60, 60));
        Grid::new(
            cx,
            ValueScaling::Linear,
            (-80., 6.),
            vec![0., -10., -20., -30., -40., -50., -60., -70.],
            Orientation::Horizontal,
        )
        .color(Color::rgb(40, 40, 40));
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
        .color(Color::rgba(255, 255, 255, 160))
        .background_color(Color::rgba(255, 255, 255, 60));
        // Displays a fade to the background color at the bottom, as a backdrop for the unit ruler
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
        .bottom(Pixels(8.));
    })
    .background_color(Color::rgb(16, 16, 16))
    .border_color(Color::rgb(80, 80, 80))
    .border_width(Pixels(1.));
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
                ValueScaling::Linear,
                (-32., 8.),
                vec![6.0, 0.0, -6.0, -12.0, -18.0, -24.0, -30.0],
                Orientation::Horizontal,
            )
            .color(Color::rgb(60, 60, 60));

            Graph::new(cx, Data::peak_buffer, (-32.0, 8.0), ValueScaling::Decibels)
                .color(Color::rgba(255, 255, 255, 160))
                .background_color(Color::rgba(255, 255, 255, 60));

            Histogram::new(cx, Data::histogram_buffer, (-32.0, 8.0))
                .color(Color::rgba(120, 120, 255, 160))
                .background_color(Color::rgba(120, 120, 255, 100))
                .width(Pixels(120.));
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
        .width(Pixels(32.));

        Meter::new(
            cx,
            Data::peak_buffer,
            (-32.0, 8.0),
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
        Grid::new(
            cx,
            ValueScaling::Linear,
            (-10., 0.),
            vec![-1., -2., -3., -4., -5., -6., -7., -8., -9.],
            Orientation::Vertical,
        )
        .color(Color::rgb(60, 60, 60));
        Grid::new(
            cx,
            ValueScaling::Linear,
            (-1.2, 1.2),
            vec![0.0, 0.5, -0.5, 1., -1.],
            Orientation::Horizontal,
        )
        .color(Color::rgb(40, 40, 40));
        Oscilloscope::new(
            cx,
            Data::oscilloscope_buffer,
            (-1.2, 1.2),
            ValueScaling::Linear,
        )
        .color(Color::rgba(255, 255, 255, 120));
    })
    .border_color(Color::rgb(80, 80, 80))
    .border_width(Pixels(1.))
    .background_color(Color::rgb(16, 16, 16));
}
