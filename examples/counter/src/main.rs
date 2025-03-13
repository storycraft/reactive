use core::pin::pin;

use reactive::{
    SetupFn, div,
    element::rect::Rect,
    event::Listener,
    pin_ref,
    skia_safe::{Color4f, Paint},
    taffy::Size,
    window::{UiWindow, ui::Ui},
    winit::event_loop::EventLoopBuilder,
};
use reactivity::let_effect;
use reactivity_winit::{run, state::StateCell};

fn main() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let _guard = rt.enter();
    run(
        EventLoopBuilder::default(),
        pin!(UiWindow::new()).into_ref().show(main_win),
    );
}

async fn main_win(ui: Ui) {
    let counter = StateCell::new(0);
    pin_ref!(counter);

    div(async move |ui: Ui| {
        let test_listener = pin!(Listener::new(|_: &mut ()| {
            counter.update(|prev| prev + 1);
        }));

        let_effect!({
            println!("counter updated to {}", counter.get($));
        });

        ui.with_style(|style| {
            style.size = Size::percent(0.3);
        });

        ui.with_mut(|mut el| {
            *el.as_mut().rect_mut() = Some({
                let mut rect = Rect::new();
                rect.fill_paint = Paint::new(Color4f::new(1.0, 1.0, 1.0, 1.0), None);
                // rect.border_radius = [Point::new(25.0, 25.0); 4];

                rect
            });
            el.as_ref().on_mouse_move().bind(test_listener);
        });

        ().show(ui).await
    })
    .show(ui)
    .await
}
