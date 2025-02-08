use core::{
    pin::{pin, Pin},
    time::Duration,
};

use futures::future::join;
use glutin_winit::DisplayBuilder;
use pin_project::pin_project;
use reactive::{render, resource::Resource, run, state::StateCell, window::SkiaWindow, Component};
use reactivity::let_effect;
use tokio::time::sleep;
use winit::{
    event::WindowEvent,
    event_loop::ActiveEventLoop,
    window::{WindowAttributes, WindowId},
};

#[tokio::main]
async fn main() {
    run(async_main());
}

async fn async_main() {
    let window = pin!(SkiaWindow::new(
        DisplayBuilder::new(),
        WindowAttributes::default()
    ));
    let window = window.into_ref();
    let tracker = pin!(MouseTracker::new());
    let tracker = tracker.into_ref();

    let input = pin!(StateCell::new(0));
    let input = input.into_ref();

    let_effect!(|| {
        if let Some(window) = &*window.window().get($) {
            println!("registered {:?}", window);
        }
    });

    let resource = Resource::new();
    let_effect!(|| {
        let input = input.get($);
        println!("Reloading resource, input: {input}");

        resource.load(async move {
            sleep(Duration::from_secs(5)).await;
            // IO heavy task
            input + 4
        });
    });

    let_effect!(|| {
        if let Some(value) = resource.get($) {
            println!("Resource loaded, value: {value}");
            input.set(value + 3);
        }
    });

    let_effect!(|| {
        let x = tracker.x().get($);
        let y = tracker.y().get($);

        println!("mouse position updated to x: {x} y: {y}");
    });

    join(render(window), render(tracker)).await;
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
