use core::pin::{pin, Pin};

use reactivity::list::{List, Node};
use scoped_tls_hkt::scoped_thread_local;

use crate::Component;

scoped_thread_local!(static COMPONENTS: for<'a> Pin<&'a List<ComponentKey>>);

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

        if COMPONENTS.is_set() {
            COMPONENTS.with(|list| {
                list.push_front(node.into_ref().entry());
            })
        }

        component.setup().await
    }

    pub fn set<R>(list: Pin<&List<ComponentKey>>, f: impl FnOnce() -> R) -> R {
        COMPONENTS.set(list, f)
    }
}
