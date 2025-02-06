use core::pin::{pin, Pin};

use futures::future::join;
use glutin_winit::DisplayBuilder;
use pin_project::pin_project;
use reactive::{let_effect, render, run, window::SkiaWindow, Component};
use reactivity::state::StateCell;
use winit::{
    event::WindowEvent,
    event_loop::ActiveEventLoop,
    window::{WindowAttributes, WindowId},
};

fn main() {
    run(async {
        let window = pin!(SkiaWindow::new(
            DisplayBuilder::new(),
            WindowAttributes::default()
        ));
        let window = window.as_ref();
        let tracker = pin!(MouseTracker::new());
        let tracker = tracker.as_ref();

        let_effect!(|| {
            if let Some(window) = &*window.window().get($) {
                println!("registered {:?}", window);
            }
        });

        let_effect!(|| {
            let x = tracker.x().get($);
            let y = tracker.y().get($);

            println!("mouse position updated to x: {x} y: {y}");
        });

        join(render(window), render(tracker)).await;
    });
}

#[derive(Debug)]
#[pin_project]
pub struct MouseTracker {
    #[pin]
    x: StateCell<f64>,
    #[pin]
    y: StateCell<f64>,
}

impl Default for MouseTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl MouseTracker {
    pub fn new() -> Self {
        Self {
            x: StateCell::new(0.0),
            y: StateCell::new(0.0),
        }
    }

    pub fn x(self: Pin<&Self>) -> Pin<&StateCell<f64>> {
        self.project_ref().x
    }

    pub fn y(self: Pin<&Self>) -> Pin<&StateCell<f64>> {
        self.project_ref().y
    }
}

impl Component<'_> for MouseTracker {
    fn on_window_event(
        self: Pin<&Self>,
        _el: &ActiveEventLoop,
        _window_id: WindowId,
        event: &mut WindowEvent,
    ) {
        if let WindowEvent::CursorMoved { position, .. } = event {
            let this = self.project_ref();
            this.x.set(position.x);
            this.y.set(position.y);
        }
    }
}
