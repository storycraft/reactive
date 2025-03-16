use core::{future::pending, pin::pin};

use futures::future::join;
use reactive::{
    SetupFn, div,
    element::rect::Rect,
    event::Listener,
    pin_ref, rotation_z,
    skia_safe::{Color4f, Paint, Point},
    styled_div,
    taffy::{self, LengthPercentage, Size, Style},
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

    let rotation = StateCell::new(0.0);
    pin_ref!(rotation);

    styled_div(
        Style {
            size: Size::percent(0.3),
            padding: taffy::Rect {
                left: LengthPercentage::Length(50.0),
                right: LengthPercentage::Length(50.0),
                top: LengthPercentage::Length(50.0),
                bottom: LengthPercentage::Length(50.0),
            },
            ..Default::default()
        },
        async move |ui: Ui| {
            let test_listener = pin!(Listener::new(|_: &mut ()| {
                counter.update(|prev| prev + 1);
            }));

            let_effect!({
                println!("counter updated to {}", counter.get($));
            });

            ui.with_mut(|mut el| {
                *el.as_mut().rect_mut() = Some({
                    let mut rect = Rect::new();
                    rect.fill_paint = Paint::new(Color4f::new(1.0, 1.0, 1.0, 1.0), None);
                    rect.border_radius = [Point::new(25.0, 25.0); 4];

                    rect
                });
                el.as_ref().on_mouse_move().bind(test_listener);
            });

            join(
                rotation_z(rotation).show(ui.clone()),
                div(async move |ui: Ui| {
                    let test_listener = pin!(Listener::new(|_: &mut ()| {
                        rotation.update(|prev| prev + 0.01);
                    }));

                    ui.with_style(|style| {
                        style.size = Size::percent(0.3);
                    });

                    ui.with_mut(|mut el| {
                        *el.as_mut().rect_mut() = Some({
                            let mut rect = Rect::new();
                            rect.fill_paint = Paint::new(Color4f::new(1.0, 1.0, 0.0, 1.0), None);
                            rect
                        });
                        el.as_ref().on_mouse_move().bind(test_listener);
                    });

                    let_effect!({
                        ui.with_mut(|mut el| {
                            el.as_mut().transform_mut().rotation.z = rotation.get($);
                        });
                    });

                    pending::<()>().await
                })
                .show(ui.clone()),
            )
            .await
            .1
        },
    )
    .show(ui)
    .await
}
