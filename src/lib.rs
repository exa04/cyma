//! Flexible, composable [VIZIA](https://github.com/vizia/vizia) views that you can
//! use to make rich [nih-plug](https://github.com/robbert-vdh/nih-plug) plug-in UIs
//! with ease.

pub mod accumulators;
pub mod bus;
pub mod spectrum;
pub mod utils;
pub mod visualizers;

pub mod prelude {
    pub use crate::{accumulators::*, bus::*, spectrum::*, utils::ValueScaling, visualizers::*};
}
