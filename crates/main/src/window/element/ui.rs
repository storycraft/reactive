use core::{
    array,
    future::Future,
    pin::{pin, Pin},
};

use pin_project::pin_project;
use reactivity::{define_safe_list, effect::Binding, list::Node, tracker::DependencyTracker};
use scopeguard::defer;
use skia_safe::Canvas;
use winit::{event::WindowEvent, event_loop::ActiveEventLoop};

use crate::event_loop::context::AppCx;

use super::{Element, SetupFn};

define_safe_list!(pub ElementList = Pin<&dyn Element>);

#[derive(Debug)]
#[pin_project]
pub struct Ui {
    #[pin]
    list: ElementList,
    #[pin]
    tracker: DependencyTracker,
}

impl Ui {
    pub fn new() -> Self {
        Self {
            list: ElementList::new(),
            tracker: DependencyTracker::new(),
        }
    }

    pub fn list(self: Pin<&Self>) -> Pin<&ElementList> {
        self.project_ref().list
    }

    fn tracker(self: Pin<&Self>) -> Pin<&DependencyTracker> {
        self.project_ref().tracker
    }

    pub fn tracked(self: Pin<&Self>, binding: Pin<&Binding>) -> Pin<&ElementList> {
        let this = self.project_ref();
        this.tracker.register(binding);
        this.list
    }

    pub async fn show<'a, const ELEMENTS: usize>(
        self: Pin<&Self>,
        elements: [Pin<&'a dyn Element>; ELEMENTS],
    ) {
        let _keys = array::from_fn::<_, ELEMENTS, _>(|i| {
            Node::new(unsafe { Pin::new_unchecked(&elements[i]) })
        });

        AppCx::with(|cx| {
            self.tracker().notify(cx.as_ref().queue());
        });

        // self.list().push_front(node.into_ref().entry());
        defer!(AppCx::with(|cx| {
            self.tracker().notify(cx.as_ref().queue());
        }));

        // element.setup().await
    }

    pub async fn add<'a, T: Element + SetupFn<'a>>(
        self: Pin<&Self>,
        element: Pin<&'a T>,
    ) -> T::Output {
        let node = pin!(Node::new(element as Pin<&'a dyn Element>));

        AppCx::with(|cx| {
            self.tracker().notify(cx.as_ref().queue());
        });

        self.list().push_front(node.into_ref().entry());
        defer!(AppCx::with(|cx| {
            self.tracker().notify(cx.as_ref().queue());
        }));

        element.setup().await
    }

    pub async fn run<'a, Fut: Future + 'a>(
        self: Pin<&'a Self>,
        f: impl FnOnce(Pin<&'a Self>) -> Fut,
    ) -> Fut::Output {
        f(self).await
    }
}

impl Default for Ui {
    fn default() -> Self {
        Self::new()
    }
}

impl Element for Ui {
    fn on_event(self: Pin<&Self>, el: &ActiveEventLoop, event: &mut WindowEvent) {
        self.list().iter(|mut iter| {
            while let Some(child) = iter.next() {
                child.value().on_event(el, event);
            }
        });
    }

    fn draw(self: Pin<&Self>, canvas: &Canvas) {
        self.list().iter(|mut iter| {
            while let Some(child) = iter.next() {
                child.value().draw(canvas);
            }
        });
    }
}
