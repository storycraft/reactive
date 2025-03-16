use core::{
    cell::{Cell, RefCell},
    pin::Pin,
};
use std::rc::Rc;

use pin_project::pin_project;
use reactive_tree::{
    ElementId,
    screen::ScreenRect,
    tree::{UiTree, element::Element},
};
use reactivity::effect::Binding;
use reactivity_winit::{
    state::StateCell,
    winit::{event::WindowEvent, window::Window},
};
use scopeguard::guard;
use taffy::Style;

#[derive(Clone)]
pub struct Ui {
    inner: Pin<Rc<Inner>>,
    current: ElementId,
}

impl Ui {
    pub fn new_root(window: Option<Window>, tree: UiTree) -> Self {
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
        let inner = &self.inner;
        if inner.draw_queued.get() {
            inner.draw_queued.set(false);
        }
        let mut tree = inner.tree.borrow_mut();
        tree.update();
        tree.draw(canvas);
    }

    pub fn dispatch_window_event(&self, event: &mut WindowEvent) {
        let mut tree = self.inner.tree.borrow_mut();

        match event {
            WindowEvent::Resized(size) => {
                tree.screen.width = size.width;
                tree.screen.height = size.height;
            }

            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                tree.screen.scale_factor = *scale_factor as _;
            }

            _ => {}
        }

        tree.window_event(event);
    }

    pub fn change_window(&self, window: Window) {
        let size = window.inner_size();
        let inner = self.inner.as_ref();

        inner.tree.borrow_mut().screen =
            ScreenRect::new(0, 0, size.width, size.height, window.scale_factor() as _);
        if inner.draw_queued.get() {
            window.request_redraw();
        }

        inner.window().set(Some(window));
    }

    pub fn close(&self) -> Option<Window> {
        self.inner.as_ref().window().take()
    }

    pub fn append(&self, style: Style) -> ElementId {
        let mut tree = self.inner.tree.borrow_mut();

        let id = tree.create(style);
        tree.append_child(self.current, id);

        id
    }

    pub fn with_ref<R>(&self, f: impl FnOnce(Pin<&Element>) -> R) -> Option<R> {
        Some(f(self.inner.tree.borrow().try_get(self.current)?))
    }

    pub fn with_mut<R>(&self, f: impl FnOnce(Pin<&mut Element>) -> R) -> Option<R> {
        Some(f(self.inner.tree.borrow_mut().try_get_mut(self.current)?))
    }

    pub fn with_style<R>(&self, f: impl FnOnce(&mut Style) -> R) -> R {
        f(self.inner.tree.borrow_mut().style_mut(self.current))
    }

    pub fn remove(&self, id: ElementId) {
        self.inner.tree.borrow_mut().destroy(id);
    }
}

#[pin_project]
struct Inner {
    #[pin]
    window: StateCell<Option<Window>>,
    draw_queued: Cell<bool>,
    tree: RefCell<UiTree>,
}

impl Inner {
    fn window(self: Pin<&Self>) -> Pin<&StateCell<Option<Window>>> {
        self.project_ref().window
    }
}
