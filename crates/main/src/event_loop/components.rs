use core::pin::{pin, Pin};

use reactivity::list::Node;

use crate::Component;

use super::context::AppCx;

pub struct ComponentKey {
    ptr: *const dyn for<'a> Component<'a>,
}

impl ComponentKey {
    pub fn with<R>(
        &self,
        f: impl for<'a> FnOnce(Pin<&'a (dyn for<'b> Component<'b> + 'a)>) -> R,
    ) -> R {
        // SAFETY: Component is pinned and guaranteed won't drop before the Node drops
        f(unsafe { Pin::new_unchecked(&*self.ptr) })
    }

    pub async fn register<T: for<'a> Component<'a>>(component: Pin<&T>) -> ! {
        let node = pin!(Node::new(ComponentKey {
            ptr: &*component as *const _ as *const _,
        }));

        if AppCx::is_set() {
            AppCx::with(|cx| {
                cx.as_ref().components().push_front(node.into_ref().entry());
            })
        }

        component.setup().await
    }
}
