#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    pub fn scale(self, intensity: f32) -> Self {
        let intensity = intensity.clamp(0.0, 1.0);
        Self::new(
            (self.r as f32 * intensity) as u8,
            (self.g as f32 * intensity) as u8,
            (self.b as f32 * intensity) as u8,
        )
    }

    pub const fn to_u32(self) -> u32 {
        ((self.r as u32) << 16) | ((self.g as u32) << 8) | self.b as u32
    }

    pub fn lerp(self, other: Self, amount: f32) -> Self {
        let amount = amount.clamp(0.0, 1.0);
        let blend = |a: u8, b: u8| (a as f32 + (b as f32 - a as f32) * amount) as u8;
        Self::new(
            blend(self.r, other.r),
            blend(self.g, other.g),
            blend(self.b, other.b),
        )
    }
}
