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
    Block, Fill,
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
    run(async_main());
}

async fn async_main() {
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

    let_effect!({
        if let Some(window) = &*win.window().get($) {
            println!("window loaded {:?}", window);
        }
    });

    win.show(|ui: Ui| async move {
        join!(
            flash_block().show(ui.clone()),
            flash_block().show(ui.clone()),
            flash_block().show(ui),
        );
    })
    .await;
}

fn flash_block<Child: SetupFn>() -> impl WithChild<Child> {
    |ui, child| async move {
        let layout = pin!(StateRefCell::new(Style {
            size: Size::from_percent(0.1, 0.1),
            ..Style::DEFAULT
        }));

        let color = pin!(StateCell::new(
            named::WHITE.into_format::<f32>().with_alpha(1.0)
        ));
        let color = color.into_ref();

        let resource = Resource::new();
        let_effect!({
            if let Some(value) = resource.get($) {
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
            .build()
            .child(child)
            .show(ui)
            .await
    }
}
