//! Views which visualize the audio running through your plug-in.

mod graph;
mod meter;

mod grid;
mod oscilloscope;
mod spectrum_analyzer;
mod unit_ruler;
mod waveform;

pub use graph::*;
pub use meter::*;

pub use grid::*;
pub use oscilloscope::*;
pub use spectrum_analyzer::*;
pub use unit_ruler::*;
pub use waveform::*;

use super::utils::ValueScaling;
use nih_plug_vizia::vizia::binding::Res;

pub trait RangeModifiers {
    /// Sets the minimum and maximum values that can be displayed by the view
    ///
    /// The values are relative to the scaling - e.g. for peak volume information,
    /// `(-48., 6.)` would be -48 to +6 dB when the scaling is set to
    /// [`ValueScaling::Decibels`]
    fn range(self, range: impl Res<(f32, f32)>) -> Self;
    /// Specifies what scaling the view should use
    fn scaling(self, scaling: impl Res<ValueScaling>) -> Self;
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
