use taffy::{Cache, Layout, Style};

#[derive(Debug)]
pub struct Node {
    pub(super) style: Style,
    pub(super) cache: Cache,
    pub(super) layout: Layout,
}

impl Node {
    pub fn new(style: Style) -> Self {
        Self {
            style,
            cache: Cache::new(),
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
}
