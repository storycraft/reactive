use nalgebra::Matrix4;
use taffy::{Cache, Layout, Style};

#[derive(Debug)]
/// Stores drawing informations
pub struct Node {
    pub style: Style,
    pub(super) cache: Cache,
    pub(super) matrix_outdated: bool,
    pub(super) matrix: Matrix4<f32>,
    pub(super) inverse_matrix: Matrix4<f32>,
    pub(super) unround_layout: Layout,
    pub(super) layout: Layout,
}

impl Node {
    pub fn new(style: Style) -> Self {
        Self {
            style,
            cache: Cache::new(),
            matrix_outdated: true,
            matrix: Matrix4::identity(),
            inverse_matrix: Matrix4::identity(),
            unround_layout: Layout::new(),
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
    pub(super) fn invalidate_matrix(&mut self) {
        if !self.matrix_outdated {
            self.matrix_outdated = true;
        }
    }

    #[inline]
    pub(super) fn cleanup(&mut self) {
        self.layout = Layout::new();
        self.matrix_outdated = true;
    }
}
