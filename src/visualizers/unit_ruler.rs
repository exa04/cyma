use nih_plug_vizia::vizia::prelude::*;
use crate::utils::ValueScaling;

/// A generic ruler.
pub struct UnitRuler {}

impl UnitRuler {
    pub fn new<'a>(
        cx: &mut Context,
        display_range: impl Res<(f32, f32)>,
        scaling: ValueScaling,
        values: impl Res<Vec<(f32, &'static str)>>,
        orientation: Orientation,
    ) -> Handle<Self> {
        Self {}.build(cx, |cx| {
            let display_range = display_range.get_val(cx);
            let normalized_values = values
                .get_val(cx)
                .into_iter()
                .map(|v| {
                    // Normalize the value according to the provided scaling, within the provided range
                    (scaling.value_to_normalized(v.0, display_range.0, display_range.1), v.1)
                })
                .collect::<Vec<(f32, &'static str)>>();
            ZStack::new(cx, |cx| {
                for value in normalized_values {
                    match orientation {
                        Orientation::Vertical => {
                            Label::new(cx, value.1)
                                .top(Percentage(100. - value.0 * 100.))
                                .transform(Transform::TranslateY(LengthOrPercentage::Percentage(
                                    -50.,
                                )));
                        }
                        Orientation::Horizontal => {
                            Label::new(cx, value.1)
                                .left(Percentage(value.0 * 100.))
                                .transform(Transform::TranslateX(LengthOrPercentage::Percentage(
                                    -50.,
                                )));
                        }
                    }
                }
            });
        })
    }
}

impl View for UnitRuler {
    fn element(&self) -> Option<&'static str> {
        Some("unit-ruler")
    }
}
