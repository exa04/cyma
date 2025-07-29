//! Generic utility functions and structures.

mod ring_buffer;
pub(crate) use ring_buffer::*;

use nih_plug::util::db_to_gain;
use nih_plug_vizia::vizia::binding::Res;
use nih_plug_vizia::vizia::context::{Context, EventContext};
use nih_plug_vizia::vizia::entity::Entity;
use nih_plug_vizia::vizia::prelude::Data;

/// Analogous to VIZIA's own ValueScaling.
#[derive(Debug, Clone, Copy, PartialEq, Data)]
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

// We can't use impl_res_simple!() since we're using nih_plug's version of VIZIA
impl Res<ValueScaling> for ValueScaling {
    fn get_val(&self, _: &Context) -> ValueScaling {
        *self
    }

    fn set_or_bind<F>(&self, cx: &mut Context, entity: Entity, closure: F)
    where
        F: 'static + Fn(&mut EventContext, Self),
    {
        cx.with_current(entity, |cx| {
            let cx = &mut EventContext::new_with_current(cx, entity);
            (closure)(cx, *self);
        });
    }
}
