//! Generic utility functions and structures.

mod ring_buffer;
pub(crate) use ring_buffer::*;

use nih_plug::util::db_to_gain;
use vizia_plug::vizia::prelude::*;

/// Analogous to VIZIA's own ValueScaling.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ValueScaling {
    Linear,
    Power(f32),
    Frequency,
    Decibels,
}

impl ValueScaling {
    pub fn normalized_to_value(&self, normalized: f32, min: f32, max: f32) -> f32 {
        let map = |x: f32| -> f32 { (x * (max - min)) + min };

        match self {
            ValueScaling::Linear => map(normalized),

            ValueScaling::Power(exponent) => map(normalized.powf(*exponent)),

            ValueScaling::Frequency => {
                let minl = min.log2();
                let range = max.log2() - minl;
                2.0f32.powf((normalized * range) + minl)
            }

            ValueScaling::Decibels => db_to_gain(normalized),
        }
    }

    pub fn value_to_normalized(&self, value: f32, min: f32, max: f32) -> f32 {
        let unmap = |x: f32| -> f32 { (x - min) / (max - min) };

        match self {
            ValueScaling::Linear => unmap(value),

            ValueScaling::Power(exponent) => unmap(value).powf(1.0 / *exponent),

            ValueScaling::Frequency => {
                let minl = min.log2();
                let range = max.log2() - minl;
                (value.log2() - minl) / range
            }

            ValueScaling::Decibels => unmap({
                const CONVERSION_FACTOR: f32 = std::f32::consts::LOG10_E * 20.0;
                value.ln() * CONVERSION_FACTOR
            }),
        }
        .clamp(0., 1.)
    }

    pub fn value_to_normalized_optional(&self, value: f32, min: f32, max: f32) -> Option<f32> {
        let unmap = |x: f32| -> f32 { (x - min) / (max - min) };

        let value = match self {
            ValueScaling::Linear => unmap(value),

            ValueScaling::Power(exponent) => unmap(value).powf(1.0 / *exponent),

            ValueScaling::Frequency => {
                let minl = min.log2();
                let range = max.log2() - minl;
                (value.log2() - minl) / range
            }

            ValueScaling::Decibels => unmap({
                const CONVERSION_FACTOR: f32 = std::f32::consts::LOG10_E * 20.0;
                value.ln() * CONVERSION_FACTOR
            }),
        };
        if (0.0..=1.0).contains(&value) {
            Some(value)
        } else {
            None
        }
    }
}

impl_res_simple!(ValueScaling);
