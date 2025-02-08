use core::{
    pin::{pin, Pin},
    time::Duration,
};

use pin_project::pin_project;
use reactive::{
    resource::Resource,
    run,
    state::StateCell,
    window::{element::Element, SkiaWindow},
};
use reactivity::let_effect;
use skia_safe::{Canvas, Color, Color4f, Paint, PaintStyle, Rect};
use tokio::time::sleep;
use winit::{event::WindowEvent, event_loop::ActiveEventLoop};

#[tokio::main]
async fn main() {
    run(async_main());
}

async fn async_main() {
    let main = pin!(SkiaWindow::new());
    let main = main.into_ref();

    let tracker = pin!(MouseTracker::new());
    let tracker = tracker.into_ref();

    let input = pin!(StateCell::new(0));
    let input = input.into_ref();

    let resource = Resource::new();
    let_effect!(|| {
        let input = input.get($);
        println!("Reloading resource due to input changes, input: {input}");

        resource.load(async move {
            // IO heavy task
            sleep(Duration::from_secs(5)).await;
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

    let_effect!(|| {
        if let Some(window) = &*main.window().get($) {
            println!("window loaded {:?}", window);
        }
    });

    main.render(|ui| async move {
        ui.add(tracker).await;
    })
    .await;
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

impl Element for MouseTracker {
    fn on_event(self: Pin<&Self>, _el: &ActiveEventLoop, event: &mut WindowEvent) {
        if let WindowEvent::CursorMoved { position, .. } = event {
            let this = self.project_ref();
            this.x.set(position.x);
            this.y.set(position.y);
        }
    }

    fn draw(self: Pin<&Self>, canvas: &Canvas) {
        let mut paint = Paint::new(Color4f::from(Color::GREEN), None);
        paint.set_style(PaintStyle::Fill);

        let x = self.x().get_untracked() as f32;
        let y = self.y().get_untracked() as f32;

        canvas.draw_rect(Rect::new(x, y, x + 50.0, y + 50.0), &paint);
    }
}
