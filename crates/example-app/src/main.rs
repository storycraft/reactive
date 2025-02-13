use core::{pin::pin, time::Duration};

use futures::join;
use rand::random_range;
use reactive::{
    taffy::{Size, Style},
    window::{ui::Ui, GuiWindow},
    SetupFn, SetupFnWithChildExt, WithChild,
};
use reactive_widgets::{
    palette::{named, rgb::channels::Argb, Srgba, WithAlpha},
    Block, Fill, Text,
};
use reactivity::let_effect;
use reactivity_winit::{
    resource::Resource,
    run,
    state::{StateCell, StateRefCell},
};
use tokio::time::sleep;
use winit::event_loop::EventLoopBuilder;

fn main() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let _guard = rt.enter();
    run(EventLoopBuilder::default(), Box::pin(async_main()));
}

pub async fn async_main() {
    let win = pin!(GuiWindow::new());
    let win = win.into_ref();

    let input = pin!(StateCell::new(0));
    let input = input.into_ref();

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

    win.show(|ui: Ui| async move {
        let effect_ui = ui.clone();
        let_effect!({
            let _ = effect_ui.with_window(|window| {
                println!("window loaded {:?}", window);
            }, $);
        });

        join!(
            flash_block()
                .child(|ui: Ui| async move {
                    join!(
                        flash_block().show(ui.clone()),
                        flash_block().show(ui.clone()),
                        flash_block().show(ui)
                    )
                })
                .show(ui.clone()),
            flash_block().show(ui.clone()),
            flash_block().show(ui),
        );
    })
    .await;
}

fn flash_block<Child: SetupFn>() -> impl WithChild<Child> {
    |ui, child| async move {
        let layout = pin!(StateRefCell::new(Style {
            size: Size::from_percent(0.3, 0.3),
            ..Style::DEFAULT
        }));

        let text = pin!(StateRefCell::new(String::new()));
        let text = text.into_ref();

        let size = pin!(StateCell::new(16.0));
        let size = size.into_ref();

        let color = pin!(StateCell::new(
            named::WHITE.into_format::<f32>().with_alpha(1.0)
        ));
        let color = color.into_ref();

        let resource = Resource::new();
        let_effect!({
            if let Some(value) = resource.get($) {
                let value: Srgba = value;
                let rgba = Srgba::<u8>::from(value);
                *text.get_mut() = format!("color: {} {} {}", rgba.red, rgba.green, rgba.blue);
                size.set(random_range(16.0..32.0));
                color.set(value);
            }

            resource.load(async move {
                sleep(Duration::from_millis(100)).await;
                Srgba::from_u32::<Argb>(random_range(0_u32..0xffffffff)).into_format()
            });
        });

        Block::builder()
            .layout(layout.into_ref())
            .fill(Fill::builder().color(color).build())
            .text(Text::builder().content(text).size(size).build())
            .build()
            .child(child)
            .show(ui)
            .await
    }
}
