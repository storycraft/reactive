use core::pin::{pin, Pin};

use futures::future::join;
use pin_project::pin_project;
use reactive::{render, run, window::Window, Component};
use reactivity::{
    effect::{binding::Binding, Effect},
    state::StateCell,
};
use winit::{
    event::WindowEvent,
    event_loop::ActiveEventLoop,
    window::{WindowAttributes, WindowId},
};

fn main() {
    run(async {
        let window = pin!(Window::new(WindowAttributes::default()));
        let window = window.as_ref();
        let tracker = pin!(Tracker::new());
        let tracker = tracker.as_ref();

        let binding = pin!(Binding::new());
        let binding = binding.as_ref();
        let effect = pin!(Effect::new(|| {
            if let Some(window) = &*window.inner().get(binding) {
                println!("registered {:?}", window);
            }
        }));
        effect.init();

        let x = pin!(Binding::new());
        let x = x.as_ref();
        let y = pin!(Binding::new());
        let y = y.as_ref();
        let effect = pin!(Effect::new(|| {
            let x = tracker.x().get(x);
            let y = tracker.y().get(y);

            println!("mouse position x: {x} y: {y}");
        }));
        effect.init();

        join(render(window), render(tracker)).await;
    });
}

#[derive(Debug)]
#[pin_project]
pub struct Tracker {
    #[pin]
    x: StateCell<f64>,
    #[pin]
    y: StateCell<f64>,
}

impl Tracker {
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

impl<'a> Component<'a> for Tracker {
    fn on_event(
        self: Pin<&'a Self>,
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
