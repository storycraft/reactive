use core::{pin::pin, time::Duration};

use futures::join;
use rand::random_range;
use reactive::{
    SetupFn, SetupFnWithChildExt, WithChild, pin_ref,
    taffy::{Size, Style},
    window::{UiWindow, ui::Ui},
    winit::event_loop::EventLoopBuilder,
};
use reactive_primitive::{
    palette::{Srgba, WithAlpha, named, rgb::channels::Argb},
    rect::{Fill, Rect},
    text::Text,
};
use reactivity::let_effect;
use reactivity_winit::{
    resource::Resource,
    run,
    state::{StateCell, StateRefCell},
};
use tokio::time::sleep;

fn main() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let _guard = rt.enter();
    run(EventLoopBuilder::default(), Box::pin(async_main()));
}

async fn async_main() {
    let input = StateCell::new(0);
    pin_ref!(input);

    let resource = Resource::new();
    let_effect!({
        let input = input.get($);
        println!("Reloading resource due to input changes, input: {input}");

        resource.load(async move {
            // IO heavy task
            sleep(Duration::from_secs(4)).await;
            input + 4
        });
    });

    let_effect!({
        if let Some(value) = resource.get($) {
            println!("Resource loaded, value: {value}");
            input.set(value + 3);
        }
    });

    pin!(UiWindow::new())
        .into_ref()
        .show(|ui: Ui| async move {
            let effect_ui = ui.clone();
            let_effect!({
                let _ = effect_ui.with_window(|window| {
                    println!("window loaded {:?}", window);
                }, $);
            });

            join!(
                flash_block
                    .child(|ui: Ui| async move {
                        join!(
                            flash_block.show(ui.clone()),
                            flash_block.show(ui.clone()),
                            flash_block.show(ui)
                        )
                    })
                    .show(ui.clone()),
                flash_block.show(ui.clone()),
                flash_block.show(ui),
            );
        })
        .await;
}

async fn flash_block<Child: SetupFn>(ui: Ui, child: Child) -> Child::Output {
    let text = StateRefCell::new(String::new());
    pin_ref!(text);

    let size = StateCell::new(16.0);
    pin_ref!(size);

    let color = StateCell::new(named::WHITE.into_format::<f32>().with_alpha(1.0));
    pin_ref!(color);

    let resource = Resource::new();
    let_effect!({
        if let Some(value) = resource.get($) {
            let value: Srgba = value;
            let rgba = Srgba::<u8>::from(value);
            text.set(format!("color: {} {} {}", rgba.red, rgba.green, rgba.blue));
            size.set(random_range(16.0..32.0));
            color.set(value);
        }

        resource.load(async move {
            sleep(Duration::from_millis(100)).await;
            Srgba::from_u32::<Argb>(random_range(0_u32..0xffffffff)).into_format()
        });
    });

    Rect::builder()
        .layout(
            pin!(StateRefCell::new(Style {
                size: Size::from_percent(0.3, 0.3),
                ..Style::DEFAULT
            }))
            .into_ref(),
        )
        .fill(Fill::builder().color(color).build())
        .build()
        .child(
            Text::builder()
                .content(text)
                .size(size)
                .build()
                .child(child),
        )
        .show(ui)
        .await
}
