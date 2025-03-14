#[derive(Debug, Clone, Default, Copy, PartialEq)]
pub enum Dimension {
    Absolute(f32),
    Percent(f32),
    #[default]
    Auto,
}

impl Dimension {
    pub fn resolve(self, v: f32) -> Option<f32> {
        match self {
            Dimension::Absolute(absolute) => Some(absolute),
            Dimension::Percent(p) => Some(p * v),
            Dimension::Auto => None,
        }
    }
}
