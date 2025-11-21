#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Rgba {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Rgba {
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    /// Convert to macroquad Color (f32 0.0-1.0)
    pub fn to_mq_color(self) -> macroquad::color::Color {
        macroquad::color::Color::from_rgba(self.r, self.g, self.b, self.a)
    }

    /// Convert from macroquad Color
    pub fn from_mq_color(c: macroquad::color::Color) -> Self {
        Self {
            r: (c.r * 255.0) as u8,
            g: (c.g * 255.0) as u8,
            b: (c.b * 255.0) as u8,
            a: (c.a * 255.0) as u8,
        }
    }
}

pub const GBA_PALETTE_ROWS: usize = 4;
pub const GBA_PALETTE_COLS: usize = 8;

pub const GBA_PALETTE: [[Rgba; GBA_PALETTE_COLS]; GBA_PALETTE_ROWS] = [
    [
        Rgba::rgb(15, 56, 15),    // dark green
        Rgba::rgb(48, 98, 48),    // mid green
        Rgba::rgb(139, 172, 15),  // yellow-green
        Rgba::rgb(155, 188, 15),  // bright yellow-green
        Rgba::rgb(62, 62, 116),   // dark blue
        Rgba::rgb(92, 92, 168),   // medium blue
        Rgba::rgb(123, 123, 213), // bright blue
        Rgba::rgb(198, 198, 198), // light gray
    ],
    [
        Rgba::rgb(247, 247, 247), // white
        Rgba::rgb(255, 188, 188), // light pink
        Rgba::rgb(255, 119, 119), // pink
        Rgba::rgb(255, 68, 68),   // hot pink/red
        Rgba::rgb(188, 63, 63),   // dark red
        Rgba::rgb(120, 0, 0),     // darker red
        Rgba::rgb(33, 30, 89),    // dark purple-blue
        Rgba::rgb(47, 50, 167),   // indigo
    ],
    [
        Rgba::rgb(0, 0, 0),       // black
        Rgba::rgb(34, 32, 52),    // very dark gray
        Rgba::rgb(69, 40, 60),    // dark brown
        Rgba::rgb(102, 57, 49),   // brown
        Rgba::rgb(143, 86, 59),   // tan
        Rgba::rgb(223, 113, 38),  // orange
        Rgba::rgb(217, 160, 102), // light tan
        Rgba::rgb(238, 195, 154), // peach
    ],
    [
        Rgba::rgb(251, 242, 54),  // bright yellow
        Rgba::rgb(153, 229, 80),  // light green
        Rgba::rgb(106, 190, 48),  // medium green
        Rgba::rgb(55, 148, 110),  // teal-green
        Rgba::rgb(75, 105, 47),   // dark green
        Rgba::rgb(82, 75, 36),    // olive
        Rgba::rgb(50, 60, 57),    // dark teal
        Rgba::rgb(63, 63, 116),   // steel blue
    ],
];
