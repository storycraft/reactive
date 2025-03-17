#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Dimension {
    Absolute(f32),
    Percent(f32),
}

impl Dimension {
    pub fn resolve(self, v: f32) -> f32 {
        match self {
            Dimension::Absolute(absolute) => absolute,
            Dimension::Percent(p) => p * v,
        }
    }
}

impl Default for Dimension {
    fn default() -> Self {
        Self::Absolute(0.0)
    }
}
