use taffy::{Cache, Layout, Style};

#[derive(Debug)]
pub struct Node {
    pub(super) style: Style,
    pub(super) cache: Cache,
    pub(super) unrounded_layout: Layout,
    pub(super) layout: Layout,
}

impl Node {
    pub fn new(style: Style) -> Self {
        Self {
            style,
            cache: Cache::new(),
            unrounded_layout: Layout::new(),
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
