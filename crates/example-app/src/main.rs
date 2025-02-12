use core::{pin::pin, time::Duration};

use rand::random_range;
use reactive::{
    taffy::{Size, Style},
    window::GuiWindow,
    SetupFnWithChildExt,
};
use reactive_widgets::{block, BlockProp};
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

    let layout = pin!(StateRefCell::new(Style::DEFAULT));
    let layout = layout.into_ref();

    let color = pin!(StateCell::new(0xffffffff));
    let color = color.into_ref();

    let input = pin!(StateCell::new(0));
    let input = input.into_ref();

    let resource = Resource::new();
    let_effect!({
        let input = input.get($);
        println!("Reloading resource due to input changes, input: {input}");

        resource.load(async move {
            // IO heavy task
            sleep(Duration::from_secs(1)).await;
            input + 4
        });
    });

    let_effect!({
        if let Some(value) = resource.get($) {
            println!("Resource loaded, value: {value}");
            input.set(value + 3);
        }

        color.set(random_range(0..0xffffffff));
        layout.get_mut().size = Size::from_lengths(random_range(50.0..300.0), random_range(50.0..300.0));
    });

    let_effect!({
        if let Some(window) = &*win.window().get($) {
            println!("window loaded {:?}", window);
        }
    });

    win.show(|ui| async move {
        block(BlockProp {
            layout: Some(layout),
            color: Some(color),
        })
        .show(ui)
        .await;
    })
    .await;
}
