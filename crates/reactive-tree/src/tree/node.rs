use nalgebra::Matrix4;
use taffy::{Cache, Layout, Style};

#[derive(Debug)]
/// Stores drawing informations
pub struct Node {
    pub(super) style: Style,
    pub(super) cache: Cache,
    pub(super) matrix: Matrix4<f32>,
    pub(super) inverse_matrix: Matrix4<f32>,
    pub(super) layout: Layout,
}

impl Node {
    pub fn new(style: Style) -> Self {
        Self {
            style,
            cache: Cache::new(),
            matrix: Matrix4::identity(),
            inverse_matrix: Matrix4::identity(),
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

    #[inline]
    pub fn inverse_matrix(&self) -> &Matrix4<f32> {
        &self.inverse_matrix
    }

    pub(super) fn cleanup(&mut self) {
        self.layout = Layout::new();
        self.matrix = Matrix4::identity();
        self.inverse_matrix = Matrix4::identity();
    }
}
