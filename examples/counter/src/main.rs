use core::{future::pending, pin::pin};

use futures::future::join;
use reactive::{
    SetupFn,
    event::Listener,
    pin_ref, rotation_z,
    skia_safe::{Color4f, Paint, Point},
    styled_div,
    taffy::{self, LengthPercentage, Size, Style},
    tree::{action::TreeActionExt, element::rect::Rect},
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
                rotation.update(|prev| prev + 0.01);
            }));

            let_effect!({
                println!("counter updated to {}", counter.get($));
            });

            ui.with_tree_mut(|tree| {
                *tree.rect_mut(ui.current_id()) = Some({
                    let mut rect = Rect::new();
                    rect.fill_paint = Paint::new(Color4f::new(1.0, 1.0, 1.0, 1.0), None);
                    rect.border_radius = [Point::new(25.0, 25.0); 4];

                    rect
                });
                tree.get(ui.current_id())
                    .as_ref()
                    .on_mouse_move()
                    .bind(test_listener);
            });

            join(
                rotation_z(rotation).show(ui.clone()),
                styled_div(
                    Style {
                        size: Size::percent(0.3),
                        ..Default::default()
                    },
                    async move |ui: Ui| {
                        ui.with_tree_mut(|tree| {
                            *tree.rect_mut(ui.current_id()) = Some({
                                let mut rect = Rect::new();
                                rect.fill_paint =
                                    Paint::new(Color4f::new(1.0, 1.0, 0.0, 1.0), None);

                                rect
                            });
                        });

                        pending::<()>().await
                    },
                )
                .show(ui.clone()),
            )
            .await
            .1
        },
    )
    .show(ui)
    .await
}
