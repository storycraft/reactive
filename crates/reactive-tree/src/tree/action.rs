use easy_ext::ext;
use taffy::Style;

use crate::{ElementId, transform::Transform};

use super::{
    UiTree,
    element::{rect::Rect, text::Text},
};

#[ext(TreeActionExt)]
pub impl UiTree {
    #[inline]
    fn style_mut(&mut self, id: ElementId) -> &mut Style {
        self.mark_dirty(id);
        &mut self.elements[id].as_mut().project().node.style
    }

    #[inline]
    fn transform_mut(&mut self, id: ElementId) -> &mut Transform {
        let project = self.elements[id].as_mut().project();
        project.node.matrix_outdated = true;
        project.transform
    }

    #[inline]
    fn rect_mut(&mut self, id: ElementId) -> &mut Option<Rect> {
        self.elements[id].as_mut().project().rect
    }

    #[inline]
    fn text_mut(&mut self, id: ElementId) -> &mut Option<Text> {
        self.elements[id].as_mut().project().text
    }
}
