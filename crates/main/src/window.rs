use core::pin::Pin;

use pin_project::pin_project;
use reactivity::state::StateRefCell;
use winit::{event_loop::ActiveEventLoop, window::WindowAttributes};

use crate::Component;

#[derive(Debug)]
#[pin_project]
pub struct Window {
    attr: WindowAttributes,
    #[pin]
    inner: StateRefCell<Option<winit::window::Window>>,
}

impl Window {
    pub fn new(attr: WindowAttributes) -> Self {
        Self {
            attr,
            inner: StateRefCell::new(None),
        }
    }

    pub fn inner(self: Pin<&Self>) -> Pin<&StateRefCell<Option<winit::window::Window>>> {
        self.project_ref().inner
    }
}

impl<'a> Component<'a> for Window {
    fn resumed(self: Pin<&'a Self>, el: &ActiveEventLoop) {
        // TODO error handling
        self.project_ref()
            .inner
            .set(Some(el.create_window(self.attr.clone()).unwrap()));
    }

    fn suspended(self: Pin<&'a Self>, _el: &ActiveEventLoop) {
        self.project_ref().inner.set(None);
    }
}
