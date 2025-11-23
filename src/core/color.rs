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
    // Row 1: Grayscale gradient (black to white)
    [
        Rgba::rgb(0, 0, 0),       // black
        Rgba::rgb(34, 32, 52),    // very dark gray
        Rgba::rgb(69, 40, 60),    // dark gray
        Rgba::rgb(102, 57, 49),   // medium dark gray
        Rgba::rgb(128, 128, 128), // mid gray
        Rgba::rgb(192, 192, 192), // light gray
        Rgba::rgb(230, 230, 230), // very light gray
        Rgba::rgb(255, 255, 255), // white
    ],
    // Row 2: Reds and oranges gradient
    [
        Rgba::rgb(120, 0, 0),     // dark red
        Rgba::rgb(188, 63, 63),   // red
        Rgba::rgb(255, 68, 68),   // bright red
        Rgba::rgb(255, 119, 119), // light red
        Rgba::rgb(255, 140, 0),   // dark orange
        Rgba::rgb(255, 165, 0),   // orange
        Rgba::rgb(255, 200, 100), // light orange
        Rgba::rgb(255, 218, 185), // peach
    ],
    // Row 3: Yellows and greens gradient
    [
        Rgba::rgb(143, 86, 59),   // brown
        Rgba::rgb(217, 160, 102), // tan
        Rgba::rgb(251, 242, 54),  // bright yellow
        Rgba::rgb(155, 188, 15),  // yellow-green
        Rgba::rgb(106, 190, 48),  // light green
        Rgba::rgb(48, 98, 48),    // mid green
        Rgba::rgb(15, 56, 15),    // dark green
        Rgba::rgb(50, 60, 57),    // dark teal
    ],
    // Row 4: Cyans, blues, and purples gradient
    [
        Rgba::rgb(55, 148, 110),  // teal
        Rgba::rgb(0, 200, 200),   // cyan
        Rgba::rgb(135, 206, 235), // sky blue
        Rgba::rgb(92, 92, 168),   // medium blue
        Rgba::rgb(62, 62, 116),   // dark blue
        Rgba::rgb(75, 0, 130),    // indigo
        Rgba::rgb(128, 0, 128),   // purple
        Rgba::rgb(255, 105, 180), // pink
    ],
];

/// Convert GBA 5-bit color component (0-31) to 8-bit (0-255)
pub fn gba5_to_u8(c5: u8) -> u8 {
    ((c5 as u16 * 255) / 31) as u8
}

/// Generate extended GBA palette with 7x7x7 = 343 colors
/// Uses evenly distributed steps through the GBA 15-bit color space
pub fn generate_gba_extended_palette() -> Vec<Rgba> {
    let steps: [u8; 7] = [0, 5, 10, 15, 20, 25, 31];
    let mut colors = Vec::new();
    for &g in &steps {
        for &r in &steps {
            for &b in &steps {
                colors.push(Rgba::rgba(gba5_to_u8(r), gba5_to_u8(g), gba5_to_u8(b), 255));
            }
        }
    }
    colors
}
