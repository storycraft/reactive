use core::pin::Pin;

use reactive::{window::ui::Ui, Element, ElementId};

pub fn use_mut<T: Element>(ui: &Ui, id: ElementId, mut f: impl FnMut(Pin<&mut T>)) {
    ui.with_mut(id, &mut f).unwrap();
}
