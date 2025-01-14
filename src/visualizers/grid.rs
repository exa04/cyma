use nih_plug_vizia::vizia::{prelude::*, vg};

use crate::utils::ValueScaling;

use super::RangeModifiers;

/// Generic grid backdrop that displays either horizontal or vertical lines.
///
/// Put this grid inside a [`ZStack`], along with your visualizer of choice.
///
/// # Example
///
/// Here's how to add a [`Grid`] as a backdrop to a [`Graph`](super::Graph).
///
/// ```
/// ZStack::new(cx, |cx| {
///     Grid::new(
///         cx,
///         ValueScaling::Linear,
///         (-32., 8.0),
///         vec![6.0, 0.0, -6.0, -12.0, -18.0, -24.0, -30.0],
///         Orientation::Horizontal,
///     )
///     .border_width(Pixels(0.5))
///     .color(Color::rgb(30, 30, 30));
///     Graph::peak(
///         cx,
///         bus.clone(),
///         10.0,
///         50.0,
///         (-32.0, 8.0),
///         ValueScaling::Decibels,
///     )
///     .color(Color::rgba(255, 255, 255, 60))
///     .background_color(Color::rgba(255, 255, 255, 30));
/// })
/// .background_color(Color::rgb(16, 16, 16))
/// .border_width(Pixels(1.0))
/// .border_color(Color::rgb(48, 48, 48));
/// ```
///
/// Note that both the `Graph` and `Grid` have the same range, which is necessary
/// for them to scale correctly.
pub struct Grid {
    scaling: ValueScaling,
    range: (f32, f32),
    lines: Vec<f32>,
    orientation: Orientation,
}

enum GridEvents {
    UpdateRange((f32, f32)),
    UpdateScaling(ValueScaling),
}

impl Grid {
    /// Creates a new [`Grid`].
    pub fn new(
        cx: &mut Context,
        scaling: ValueScaling,
        range: impl Res<(f32, f32)>,
        lines: impl Res<Vec<f32>>,
        orientation: Orientation,
    ) -> Handle<Self> {
        Self {
            scaling,
            range: range.get_val(cx),
            lines: lines.get_val(cx),
            orientation,
        }
        .build(cx, |_| {})
        .range(range)
        .scaling(scaling)
    }
}

impl View for Grid {
    fn element(&self) -> Option<&'static str> {
        Some("grid")
    }
    fn draw(&self, cx: &mut DrawContext, canvas: &mut Canvas) {
        let bounds = cx.bounds();

        let x = bounds.x;
        let y = bounds.y;
        let w = bounds.w;
        let h = bounds.h;

        let line_width = if cx.border_width() > 0.0 {
            cx.border_width() * cx.scale_factor()
        } else {
            cx.scale_factor()
        };

        canvas.stroke_path(
            &{
                let mut path = vg::Path::new();

                match self.orientation {
                    Orientation::Horizontal => {
                        for y_line in self.lines.iter() {
                            let y_line = self.scaling.value_to_normalized(
                                *y_line,
                                self.range.0,
                                self.range.1,
                            );

                            path.move_to(x, y + h * (1. - y_line));
                            path.line_to(x + w, y + h * (1. - y_line));

                            path.close();
                        }
                    }
                    Orientation::Vertical => {
                        for x_line in self.lines.iter() {
                            let x_line = self.scaling.value_to_normalized(
                                *x_line,
                                self.range.0,
                                self.range.1,
                            );

                            path.move_to(x + w * x_line, y);
                            path.line_to(x + w * x_line, y + h);

                            path.close();
                        }
                    }
                };

                path
            },
            &vg::Paint::color(cx.font_color().into()).with_line_width(line_width),
        );
    }
    fn event(&mut self, _cx: &mut EventContext, event: &mut Event) {
        event.map(|e, _| match e {
            GridEvents::UpdateRange(v) => self.range = *v,
            GridEvents::UpdateScaling(v) => self.scaling = *v,
        });
    }
}

impl<'a> RangeModifiers for Handle<'a, Grid> {
    fn range(mut self, range: impl Res<(f32, f32)>) -> Self {
        let e = self.entity();

        range.set_or_bind(self.context(), e, move |cx, r| {
            (*cx).emit_to(e, GridEvents::UpdateRange(r.clone()));
        });

        self
    }
    fn scaling(mut self, scaling: impl Res<ValueScaling>) -> Self {
        let e = self.entity();

        scaling.set_or_bind(self.context(), e, move |cx, s| {
            (*cx).emit_to(e, GridEvents::UpdateScaling(s));
        });

        self
    }
}
