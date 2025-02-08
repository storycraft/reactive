use core::{
    future::{pending, Future},
    pin::{pin, Pin},
};

use pin_project::pin_project;
use reactivity::list::{List, Node};
use skia_safe::Canvas;
use winit::{event::WindowEvent, event_loop::ActiveEventLoop};

use super::Element;

#[derive(Debug)]
#[pin_project]
pub struct Children {
    #[pin]
    list: List<ElementKey>,
}

impl Children {
    pub fn new() -> Self {
        Self { list: List::new() }
    }

    fn list(self: Pin<&Self>) -> Pin<&List<ElementKey>> {
        self.project_ref().list
    }

    pub async fn add(self: Pin<&Self>, element: Pin<&dyn Element>) -> ! {
        let node = pin!(Node::new(ElementKey {
            ptr: &*element as *const _ as *const _,
        }));

        self.list().push_front(node.into_ref().entry());
        pending().await
    }

    pub async fn run<'a, Fut: Future<Output = ()> + 'a>(
        self: Pin<&'a Self>,
        f: impl FnOnce(Pin<&'a Children>) -> Fut,
    ) {
        f(self).await;
    }
}

impl Default for Children {
    fn default() -> Self {
        Self::new()
    }
}

impl Element for Children {
    fn on_event(self: Pin<&Self>, el: &ActiveEventLoop, event: &mut WindowEvent) {
        for child in self.list().iter() {
            child.value().with(|child| {
                child.on_event(el, event);
            });
        }
    }

    fn draw(self: Pin<&Self>, canvas: &Canvas) {
        for child in self.list().iter() {
            child.value().with(|child| {
                child.draw(canvas);
            });
        }
    }
}

#[derive(Debug)]
pub struct ElementKey {
    ptr: *const dyn Element,
}

impl ElementKey {
    pub fn with<R>(&self, f: impl FnOnce(Pin<&dyn Element>) -> R) -> R {
        // SAFETY: Component is pinned and guaranteed won't drop before the Node drops
        f(unsafe { Pin::new_unchecked(&*self.ptr) })
    }
}
