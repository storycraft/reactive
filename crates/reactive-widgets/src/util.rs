use core::pin::Pin;

use reactive::{window::ui::Ui, Element, ElementId};

pub fn use_mut<'a, T: Element>(
    ui: Ui<'a>,
    id: ElementId,
    mut f: impl FnMut(Pin<&mut T>) + 'a,
) {
    ui.with_mut(id, &mut f).unwrap();
}
