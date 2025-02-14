pub use reactivity;

use core::pin::Pin;

use reactive::{window::ui::Ui, Element, ElementId};

pub fn use_mut<T: Element>(ui: &Ui, id: ElementId, mut f: impl FnMut(Pin<&mut T>)) {
    ui.with_mut(id, &mut f).unwrap();
}

macro_rules! create_wire_macro {
    ($macro_name:ident, $ui:expr, $id:expr) => {
        $crate::util::create_wire_macro!(($) $macro_name, $ui, $id)
    };

    (($d:tt) $macro_name:ident, $ui:expr, $id:expr) => {
        macro_rules! $macro_name {
            ($name:pat = $prop:expr => $d($tt:tt)*) => {
                $crate::util::reactivity::let_effect!({
                    if let Some($name) = $prop {
                        $ui.request_redraw();
        
                        $d($tt)*
                    }
                });
            };
        
            ($element:ident: $ty:ty, $name:pat = $prop:expr => $d($tt:tt)*) => {
                $macro_name!($name = $prop => {
                    $crate::util::use_mut(
                        &$ui,
                        $id,
                        |#[allow(unused_mut)] mut $element: ::core::pin::Pin<&mut $ty>| {
                            $d($tt)*
                        }
                    )
                });
            };
        }
    }
}

pub(crate) use create_wire_macro;