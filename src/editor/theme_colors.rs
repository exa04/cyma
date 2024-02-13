use nih_plug_vizia::vizia::style::Color;

pub struct ThemeColors {}
impl ThemeColors {
    pub const FOREGROUND: Color = Color::rgb(0, 0, 0);
    pub const FOREGROUND_FADED: Color = Color::rgba(0, 0, 0, 80);
    pub const BACKGROUND: Color = Color::rgb(209, 213, 219);
    pub const ACCENT: Color = Color::rgb(0, 0, 255);
    pub const ACCENT_FADED: Color = Color::rgba(0, 0, 255, 80);
}
