//! Views which visualize the audio running through your plug-in.

mod graph;
mod grid;
mod lissajous;
mod meter;
mod oscilloscope;
mod spectrum_analyzer;
mod unit_ruler;
mod waveform;

pub use graph::*;
pub use grid::*;
pub use lissajous::*;
pub use meter::*;
pub use oscilloscope::*;
pub use spectrum_analyzer::*;
pub use unit_ruler::*;
pub use waveform::*;

use nih_plug_vizia::vizia::binding::Res;

pub trait RangeModifiers {
    fn range(self, range: impl Res<(f32, f32)>) -> Self;
}
