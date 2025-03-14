use nalgebra::Matrix4;
use taffy::{Cache, Layout, Style};

#[derive(Debug)]
pub struct Node {
    pub(super) style: Style,
    pub(super) cache: Cache,
    pub(super) matrix: Matrix4<f32>,
    pub(super) layout: Layout,
}

impl Node {
    pub fn new(style: Style) -> Self {
        Self {
            style,
            cache: Cache::new(),
            matrix: Matrix4::identity(),
            layout: Layout::new(),
        }
    }

    #[inline]
    pub fn style(&self) -> &Style {
        &self.style
    }

    #[inline]
    pub fn layout(&self) -> &Layout {
        &self.layout
    }

    #[inline]
    pub fn matrix(&self) -> &Matrix4<f32> {
        &self.matrix
    }
}
