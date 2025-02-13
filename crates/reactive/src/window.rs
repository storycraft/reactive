mod state;
pub mod ui;

use core::{cell::RefCell, num::NonZeroU32, pin::Pin};
use glutin_winit::DisplayBuilder;
use reactivity_winit::{
    event_loop::handler::{self, WinitWindow},
    winit::{
        event::WindowEvent,
        event_loop::ActiveEventLoop,
        window::{WindowAttributes, WindowId},
    },
};
use skia_safe::Color;
use state::{Context, WindowState};
use ui::Ui;

use crate::{tree::Tree, SetupFn};

pub struct GuiWindow {
    attr: WindowAttributes,
    state: RefCell<WindowState>,
    ui: Ui,
}

impl GuiWindow {
    pub fn new() -> Self {
        let attr = WindowAttributes::default();
        let builder = DisplayBuilder::new().with_window_attributes(Some(attr.clone()));

        Self {
            state: RefCell::new(WindowState::new(builder)),
            attr,
            ui: Ui::new_root(None, Tree::new()),
        }
    }

    pub async fn show<F: SetupFn>(self: Pin<&Self>, f: F) -> F::Output {
        handler::add(self, f.show(self.ui.clone())).await
    }
}

impl Default for GuiWindow {
    fn default() -> Self {
        Self::new()
    }
}

impl WinitWindow for GuiWindow {
    fn window_id(self: Pin<&Self>) -> Option<WindowId> {
        let WindowState::Init(Context { id, .. }) = *self.state.borrow() else {
            return None;
        };

        Some(id)
    }

    fn resumed(self: Pin<&Self>, el: &ActiveEventLoop) {
        // TODO:: error handling
        let Some(window) = self.state.borrow_mut().create_window(el, &self.attr) else {
            panic!("window creation failed")
        };

        self.ui.change_window(window);
    }

    fn suspended(self: Pin<&Self>, _el: &ActiveEventLoop) {
        self.state.borrow_mut().suspend();
        self.ui.close();
    }

    fn on_window_event(self: Pin<&Self>, el: &ActiveEventLoop, event: &mut WindowEvent) {
        let WindowState::Init(cx) = &mut *self.state.borrow_mut() else {
            return;
        };

        match event {
            WindowEvent::Resized(size) => {
                if let (Some(width), Some(height)) =
                    (NonZeroU32::new(size.width), NonZeroU32::new(size.height))
                {
                    cx.resize(width, height);
                    self.ui.resize(width.get(), height.get());
                }
            }

            WindowEvent::RedrawRequested => {
                let canvas = cx.canvas();
                canvas.clear(Color::BLACK);
                self.ui.draw(canvas);
                cx.render();
            }

            _ => {}
        }

        self.ui.dispatch_window_event(el, event);
    }
}
