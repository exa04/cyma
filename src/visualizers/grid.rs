use nih_plug_vizia::vizia::{
    binding::Res,
    context::{Context, DrawContext},
    vg,
    view::{Canvas, Handle, View},
    views::Orientation,
};

use crate::utils::ValueScaling;

/// A generic grid backdrop that displays either horizontal or vertical lines.
///
/// Put this grid inside a ZStack, along with your visualizer of choice.
pub struct Grid {
    scaling: ValueScaling,
    range: (f32, f32),
    lines: Vec<f32>,
    orientation: Orientation,
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
}
