use nih_plug_vizia::vizia::{
    binding::Res,
    context::{Context, DrawContext},
    vg,
    view::{Canvas, Handle, View},
};

/// A generic grid backdrop.
///
/// Put this grid inside a ZStack, along with your visualizer of choice.
pub struct Grid {
    display_range: (f32, f32),
    x_subdivisions: f32,
    y_lines: Vec<f32>,
}

impl Grid {
    pub fn new(
        cx: &mut Context,
        display_range: impl Res<(f32, f32)>,
        x_subdivisions: impl Res<f32>,
        y_lines: impl Res<Vec<f32>>,
    ) -> Handle<Self> {
        Self {
            display_range: display_range.get_val(cx),
            x_subdivisions: x_subdivisions.get_val(cx),
            y_lines: y_lines.get_val(cx),
        }
        .build(cx, |_| {})
    }
}

impl View for Grid {
    fn element(&self) -> Option<&'static str> {
        None
    }
    fn draw(&self, cx: &mut DrawContext, canvas: &mut Canvas) {
        let bounds = cx.bounds();

        let x = bounds.x;
        let y = bounds.y;
        let w = bounds.w;
        let h = bounds.h;

        let line_width = cx.scale_factor();

        // Horizontal grid lines
        canvas.stroke_path(
            &{
                let mut path = vg::Path::new();

                for y_line in self.y_lines.iter() {
                    // Clamp y value in range
                    let mut y_line = y_line.clamp(self.display_range.0, self.display_range.1);

                    // Normalize peak's range
                    y_line -= self.display_range.0;
                    y_line /= self.display_range.1 - self.display_range.0;

                    // Draw a line at y from left to right
                    path.move_to(x, y + h * (1. - y_line));
                    path.line_to(x + w, y + h * (1. - y_line));

                    path.close();
                }

                path
            },
            &vg::Paint::color(cx.font_color().into()).with_line_width(line_width),
        );

        if self.x_subdivisions == 0.0 {
            return;
        }

        // Horizontal grid lines
        canvas.stroke_path(
            &{
                let mut path = vg::Path::new();

                let t_delta = w / self.x_subdivisions;

                for step in (0..self.x_subdivisions.ceil() as u32).map(|x| x as f32 * t_delta) {
                    path.move_to(x + w - step, y);
                    path.line_to(x + w - step, y + h);
                    path.close();
                }

                path
            },
            &vg::Paint::color(cx.font_color().into()).with_line_width(line_width),
        );
    }
}
