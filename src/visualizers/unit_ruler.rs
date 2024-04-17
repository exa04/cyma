use crate::utils::ValueScaling;
use nih_plug_vizia::vizia::prelude::*;

/// A generic ruler.
///
/// Takes in a display range and scaling, as well as values within that range, where
/// unit markers will be displayed.
/// 
/// ```
/// UnitRuler::new(
///     cx,
///     (-32.0, 8.0),
///     ValueScaling::Linear,
///     vec![
///         (6.0, "6db"),
///         (0.0, "0db"),
///         (-6.0, "-6db"),
///         (-12.0, "-12db"),
///         (-18.0, "-18db"),
///         (-24.0, "-24db"),
///         (-30.0, "-30db"),
///     ],
///     Orientation::Vertical,
/// )
/// .font_size(12.)
/// .color(Color::rgb(160, 160, 160))
/// .width(Pixels(32.))
/// .height(Pixels(128.));
/// ```
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
                    (
                        scaling.value_to_normalized(v.0, display_range.0, display_range.1),
                        v.1,
                    )
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
