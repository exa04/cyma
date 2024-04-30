use nih_plug_vizia::vizia::{prelude::*, vg};

use crate::utils::ValueScaling;

use super::RangeModifiers;

/// Generic grid backdrop that displays either horizontal or vertical lines.
///
/// Put this grid inside a ZStack, along with your visualizer of choice.
///
/// # Example
///
/// Here's how to add a `Grid` as a backdrop to a `Graph`.
///
/// ```
/// ZStack::new(cx, |cx| {
///     Grid::new(
///         cx,
///         ValueScaling::Linear,
///         (-32., 8.),
///         vec![6.0, 0.0, -6.0, -12.0, -18.0, -24.0, -30.0],
///         Orientation::Horizontal,
///     )
///     .color(Color::rgb(60, 60, 60));
///
///     Graph::new(cx, Data::peak_buffer, (-32.0, 8.0), ValueScaling::Decibels)
///         .color(Color::rgba(255, 255, 255, 160))
///         .background_color(Color::rgba(255, 255, 255, 60));
/// })
/// .background_color(Color::rgb(16, 16, 16));
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
}

impl Grid {
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

        let line_width = cx.scale_factor();

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
    fn event(
        &mut self,
        _cx: &mut nih_plug_vizia::vizia::context::EventContext,
        event: &mut nih_plug_vizia::vizia::events::Event,
    ) {
        event.map(|e, _| match e {
            GridEvents::UpdateRange(v) => self.range = *v,
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
}
