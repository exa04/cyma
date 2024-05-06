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

pub(crate) enum FillFrom {
    Top,
    Bottom,
    Value(f32),
}

pub trait FillModifiers {
    /// Allows for the view to be filled from the max instead of the min value.
    fn fill_from_max(self) -> Self;

    /// Allows for the view to be filled from any desired level.
    fn fill_from_value(self, level: f32) -> Self;
}
