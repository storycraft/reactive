use core::{
    cell::{Cell, RefCell},
    pin::Pin,
};
use std::rc::Rc;

use pin_project::pin_project;
use reactivity::effect::Binding;
use reactivity_winit::{
    state::StateCell,
    winit::{event::WindowEvent, event_loop::ActiveEventLoop, window::Window},
};
use scopeguard::guard;
use taffy;

use crate::{Element, ElementId, tree::Tree};

#[derive(Clone)]
pub struct Ui {
    inner: Pin<Rc<Inner>>,
    current: ElementId,
}

impl Ui {
    pub fn new_root(window: Option<Window>, tree: Tree) -> Self {
        let current = tree.root();

        Self {
            inner: Rc::pin(Inner {
                window: StateCell::new(window),
                draw_queued: Cell::new(false),
                tree: RefCell::new(tree),
            }),
            current,
        }
    }

    pub fn sub_ui(&self, child: ElementId) -> Ui {
        Self {
            inner: self.inner.clone(),
            current: child,
        }
    }

    pub fn current_id(&self) -> ElementId {
        self.current
    }

    #[must_use]
    pub fn with_window<R>(&self, f: impl FnOnce(&Window) -> R, binding: Binding) -> Option<R> {
        let cell = self.inner.as_ref().window();
        let window = guard(cell.take_get(binding)?, |window| {
            cell.set_untracked(Some(window))
        });

        Some(f(&window))
    }

    #[must_use]
    pub fn with_window_untracked<R>(&self, f: impl FnOnce(&Window) -> R) -> Option<R> {
        let cell = self.inner.as_ref().window();
        let window = guard(cell.take_get_untracked()?, |window| {
            cell.set_untracked(Some(window))
        });

        Some(f(&window))
    }

    pub fn resize(&self, width: u32, height: u32) {
        self.inner.tree.borrow_mut().resize(width, height);
    }

    pub fn request_layout(&self) {
        self.inner.tree.borrow_mut().mark_dirty(self.current);
    }

    pub fn request_redraw(&self) {
        if !self.inner.draw_queued.get() {
            self.inner.draw_queued.set(true);
            _ = self.with_window_untracked(|window| window.request_redraw());
        }
    }

    pub fn draw(&self, canvas: &skia_safe::Canvas) {
        let inner = self.inner.as_ref();
        if inner.draw_queued.get() {
            inner.draw_queued.set(false);
        }
        inner.tree.borrow_mut().draw(canvas);
    }

    pub fn dispatch_window_event(&self, el: &ActiveEventLoop, event: &mut WindowEvent) {
        self.inner.tree.borrow_mut().window_event(el, event);
    }

    pub fn change_window(&self, window: Window) {
        let size = window.inner_size();
        let inner = self.inner.as_ref();

        inner.tree.borrow_mut().resize(size.width, size.height);
        if inner.draw_queued.get() {
            window.request_redraw();
        }

        inner.window().set(Some(window));
    }

    pub fn close(&self) -> Option<Window> {
        self.inner.as_ref().window().take()
    }

    pub fn append<T>(&self, layout: taffy::Style, element: T) -> ElementId
    where
        T: Element + 'static,
    {
        let mut tree = self.inner.tree.borrow_mut();
        let id = tree.create(layout, element);
        tree.append(self.current, id);
        id
    }

    #[must_use]
    pub fn with_ref<T: Element, R>(
        &self,
        id: ElementId,
        f: impl FnOnce(Pin<&T>) -> R,
    ) -> Option<R> {
        Some(f(self.inner.tree.borrow().get(id)?))
    }

    #[must_use]
    pub fn with_mut<T: Element, R>(
        &self,
        id: ElementId,
        f: impl FnOnce(Pin<&mut T>) -> R,
    ) -> Option<R> {
        Some(f(self.inner.tree.borrow_mut().get_mut(id)?))
    }

    pub fn remove_child(&self, id: ElementId) {
        self.inner.tree.borrow_mut().remove_child(self.current, id)
    }

    pub fn remove(&self, id: ElementId) {
        self.inner.tree.borrow_mut().remove(id)
    }

    pub fn set_style(&self, style: taffy::Style) {
        self.inner.tree.borrow_mut().set_style(self.current, style);
    }
}

#[pin_project]
struct Inner {
    #[pin]
    window: StateCell<Option<Window>>,
    draw_queued: Cell<bool>,
    tree: RefCell<Tree>,
}

impl Inner {
    fn window(self: Pin<&Self>) -> Pin<&StateCell<Option<Window>>> {
        self.project_ref().window
    }
}
