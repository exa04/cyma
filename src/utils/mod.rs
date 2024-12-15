//! Generic utility functions and structures.

pub mod accumulators;
pub mod buffers;
mod channel;
mod ring_buffer;
mod spectrum;

pub use channel::*;
pub use ring_buffer::*;
pub use spectrum::*;

use nih_plug::util::db_to_gain;
use nih_plug_vizia::vizia::binding::Res;
use nih_plug_vizia::vizia::context::{Context, EventContext};
use nih_plug_vizia::vizia::entity::Entity;

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
        if normalized <= 0.0 {
            return min;
        } else if normalized >= 1.0 {
            return max;
        }

        let map = |x: f32| -> f32 { (x * (max - min)) + min };

        match self {
            ValueScaling::Linear => map(normalized),

            ValueScaling::Power(exponent) => map(normalized.powf(*exponent)),

            ValueScaling::Frequency => {
                let minl = min.log2();
                let range = max.log2() - minl;
                2.0f32.powf((normalized * range) + minl)
            }

            ValueScaling::Decibels => map(db_to_gain(normalized)),
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
/*
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ValueScaling {
    Linear { min: f32, max: f32 },
    Power { min: f32, max: f32, exponent: f32 },
    Frequency { min: f32, max: f32 },
    Decibels { min: f32, max: f32 },
}
impl ValueScaling {
    #[inline]
    fn map(x: f32, min: &f32, max: &f32) -> f32 {
        (x * (max - min)) + min
    }
    #[inline]
    fn unmap(x: f32, min: &f32, max: &f32) -> f32 {
        (x - min) / (max - min)
    }
    pub fn normalized_to_value(&self, normalized: f32) -> f32 {
        match self {
            ValueScaling::Linear { min, max } => {
                if normalized <= 0.0 {
                    return *min;
                } else if normalized >= 1.0 {
                    return *max;
                }
                Self::map(normalized, min, max)
            }
            ValueScaling::Power { min, max, exponent } => {
                if normalized <= 0.0 {
                    return *min;
                } else if normalized >= 1.0 {
                    return *max;
                }
                Self::map(normalized.powf(*exponent), min, max)
            }
            ValueScaling::Frequency { min, max } => {
                if normalized <= 0.0 {
                    return *min;
                } else if normalized >= 1.0 {
                    return *max;
                }

                let minl = min.log2();
                let range = max.log2() - minl;
                2.0f32.powf((normalized * range) + minl)
            }
            ValueScaling::Decibels { min, max } => {
                if normalized <= 0.0 {
                    return *min;
                } else if normalized >= 1.0 {
                    return *max;
                }
                Self::map(db_to_gain(normalized), min, max)
            }
        }
    }

    pub fn value_to_normalized(&self, value: f32) -> f32 {
        match self {
            ValueScaling::Linear { min, max } => Self::unmap(value, min, max),
            ValueScaling::Power { min, max, exponent } => {
                Self::unmap(value, min, max).powf(1.0 / *exponent)
            }
            ValueScaling::Frequency { min, max } => {
                let minl = min.log2();
                let range = max.log2() - minl;
                (value.log2() - minl) / range
            }
            ValueScaling::Decibels { min, max } => Self::unmap(
                value.ln() * const { std::f32::consts::LOG10_E * 20.0 },
                min,
                max,
            ),
        }
        .clamp(0., 1.)
    }

    pub fn value_to_normalized_optional(&self, value: f32) -> Option<f32> {
        let value = match self {
            ValueScaling::Linear { min, max } => Self::unmap(value, min, max),

            ValueScaling::Power { min, max, exponent } => {
                Self::unmap(value, min, max).powf(1.0 / *exponent)
            }

            ValueScaling::Frequency { min, max } => {
                let minl = min.log2();
                let range = max.log2() - minl;
                (value.log2() - minl) / range
            }

            ValueScaling::Decibels { min, max } => Self::unmap(
                value.ln() * const { std::f32::consts::LOG10_E * 20.0 },
                min,
                max,
            ),
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
            closure(cx, *self);
        });
    }
}
*/
