mod state;
pub mod ui;

use core::{
    cell::RefCell,
    num::NonZeroU32,
    pin::{pin, Pin},
};
use glutin_winit::DisplayBuilder;
use pin_project::pin_project;
use reactivity_winit::{
    event_loop::handler::{self, WinitWindow},
    state::StateRefCell,
    winit::{
        event::WindowEvent,
        event_loop::ActiveEventLoop,
        window::{Window, WindowAttributes, WindowId},
    },
};
use skia_safe::Color;
use state::{Context, WindowState};
use std::rc::Rc;
use ui::Ui;

use crate::{tree::Tree, SetupFn};

#[derive(Debug)]
#[pin_project]
pub struct GuiWindow {
    attr: WindowAttributes,
    state: RefCell<WindowState>,
    #[pin]
    window: StateRefCell<Option<Window>>,
    ui: Rc<RefCell<Tree>>,
}

impl GuiWindow {
    pub fn new() -> Self {
        let attr = WindowAttributes::default();
        let builder = DisplayBuilder::new().with_window_attributes(Some(attr.clone()));

        Self {
            state: RefCell::new(WindowState::new(builder)),
            attr,
            window: StateRefCell::new(None),
            ui: Rc::new(RefCell::new(Tree::new())),
        }
    }

    pub fn window(self: Pin<&Self>) -> Pin<&StateRefCell<Option<Window>>> {
        self.project_ref().window
    }

    pub async fn show<F: SetupFn>(self: Pin<&Self>, f: F) -> F::Output {
        handler::add(self, f.show(Ui::root(self.get_ref().ui.clone()))).await
    }
}

impl Default for GuiWindow {
    fn default() -> Self {
        Self::new()
    }
}

impl WinitWindow for GuiWindow {
    fn window_id(self: Pin<&Self>) -> Option<WindowId> {
        let WindowState::Init(Context { id, .. }) = *self.project_ref().state.borrow() else {
            return None;
        };

        Some(id)
    }

    fn request_redraw(self: Pin<&Self>) {
        if let Some(window) = &*self.project_ref().window.get_untracked() {
            window.request_redraw();
        }
    }

    fn resumed(self: Pin<&Self>, el: &ActiveEventLoop) {
        let this = self.project_ref();

        // TODO:: error handling
        let Some(window) = this.state.borrow_mut().create_window(el, &self.attr) else {
            panic!("window creation failed")
        };

        this.window.set(Some(window));
    }

    fn suspended(self: Pin<&Self>, _el: &ActiveEventLoop) {
        let this = self.project_ref();
        this.state.borrow_mut().suspend();
        this.window.set(None);
    }

    fn on_window_event(self: Pin<&Self>, el: &ActiveEventLoop, event: &mut WindowEvent) {
        let this = self.project_ref();

        let WindowState::Init(cx) = &mut *this.state.borrow_mut() else {
            return;
        };

        match event {
            WindowEvent::Resized(size) => {
                if let (Some(width), Some(height)) =
                    (NonZeroU32::new(size.width), NonZeroU32::new(size.height))
                {
                    cx.resize(width, height);
                    this.ui.borrow_mut().resize(width.get(), height.get());
                }
            }

            WindowEvent::RedrawRequested => {
                let canvas = cx.canvas();
                canvas.clear(Color::BLACK);
                this.ui.borrow_mut().redraw(canvas);
                cx.render();
            }

            _ => {}
        }

        this.ui.borrow().window_event(el, event);
    }
}
