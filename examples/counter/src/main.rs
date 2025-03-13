use core::{future::pending, pin::pin};

use reactive::{
    SetupFn, create_element,
    element::rect::Rect,
    event::Listener,
    pin_ref,
    skia_safe::{Color4f, Paint},
    taffy::{Size, Style},
    window::{UiWindow, ui::Ui},
    winit::event_loop::EventLoopBuilder,
};
use reactivity_winit::{run, state::StateCell};

fn main() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let _guard = rt.enter();
    run(EventLoopBuilder::default(), ui_main());
}

async fn ui_main() {
    let counter = StateCell::new(0);
    pin_ref!(counter);

    pin!(UiWindow::new())
        .into_ref()
        .show(async move |ui| {
            create_element(
                Style {
                    size: Size::from_percent(0.3, 0.3),
                    ..Style::DEFAULT
                },
                async move |ui: Ui| {
                    let test_listener = pin!(Listener::new(|_: &mut ()| {
                        println!("mouse_move event test && hittest");
                    }));

                    _ = ui.with_mut(ui.current_id(), |mut el| {
                        *el.as_mut().rect_mut() = Some({
                            let mut rect = Rect::new();
                            rect.fill_paint = Paint::new(Color4f::new(1.0, 1.0, 1.0, 1.0), None);
                            // rect.border_radius = [Point::new(25.0, 25.0); 4];

                            rect
                        });
                        el.as_ref().on_mouse_move().bind(test_listener);
                    });

                    pending::<()>().await
                },
            )
            .show(ui)
            .await
        })
        .await;
}
