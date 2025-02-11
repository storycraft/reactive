use core::{
    cell::Cell,
    future::pending,
    pin::{pin, Pin},
    time::Duration,
};

use rand::random_range;
use reactive::{
    taffy::{Dimension, Size, Style},
    window::GuiWindow,
    with_children, wrap_element, Element, SetupFn, SetupFnWithChild, SetupFnWithChildExt,
};
use reactivity::let_effect;
use reactivity_winit::{
    resource::Resource,
    run,
    state::{StateCell, StateRefCell},
};
use skia_safe::{Canvas, Color4f, Paint, PaintStyle, Rect};
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

        color.set(random_range(0..0xffffffff));
        layout.get_mut().size = Size::from_lengths(random_range(50.0..300.0), random_range(50.0..300.0));
    });

    let_effect!(|| {
        if let Some(window) = &*win.window().get($) {
            println!("window loaded {:?}", window);
        }
    });

    win.show(|ui| async move {
        block(BlockProp { layout, color }).show(ui).await;
    })
    .await;
}

pub struct BlockProp<'a> {
    layout: Pin<&'a StateRefCell<Style>>,
    color: Pin<&'a StateCell<u32>>,
}

pub fn block<'a, Child: SetupFn<'a>>(prop: BlockProp<'a>) -> impl SetupFnWithChild<'a, Child> {
    with_children::<Child, _>(move |child| {
        wrap_element(
            Style {
                size: Size {
                    width: Dimension::Percent(0.25),
                    height: Dimension::Percent(0.25),
                },
                ..Default::default()
            },
            Block::new(),
            move |ui, element| async move {
                let_effect!(|| {
                    element.color.set(prop.color.get($));
                });

                let_effect!(|| {
                    ui.set_style(ui.current(), prop.layout.get($).clone());
                });

                child.show(ui).await;
                pending::<()>().await;
            },
        )
    })
}

#[derive(Debug)]
pub struct Block {
    pub color: Cell<u32>,
}

impl Block {
    pub fn new() -> Self {
        Self {
            color: Cell::new(0xffffffff),
        }
    }
}

impl Element for Block {
    fn draw(self: Pin<&Self>, canvas: &Canvas, width: f32, height: f32) {
        let mut paint = Paint::new(Color4f::from(self.color.get()), None);
        paint.set_style(PaintStyle::Fill);

        canvas.draw_rect(Rect::new(0.0, 0.0, width, height), &paint);
    }
}
