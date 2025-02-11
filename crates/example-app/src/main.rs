use core::{
    cell::Cell,
    future::pending,
    pin::{pin, Pin},
    time::Duration,
};

use futures::join;
use rand::random_range;
use reactive::{
    taffy::{Dimension, Size, Style}, window::GuiWindow, with_children, wrap_element, Element, SetupFn,
    SetupFnWithChild, SetupFnWithChildExt,
};
use reactivity::let_effect;
use reactivity_winit::{resource::Resource, run, state::StateCell};
use skia_safe::{Canvas, Color, Color4f, Paint, PaintStyle, Rect};
use tokio::time::sleep;

fn main() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let _guard = rt.enter();
    run(async_main());
}

async fn async_main() {
    let win = pin!(GuiWindow::new());
    let win = win.into_ref();

    let x = pin!(StateCell::new(0.0));
    let x = x.into_ref();

    let y = pin!(StateCell::new(0.0));
    let y = y.into_ref();

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

        x.set(random_range(0.0..800.0));
        y.set(random_range(0.0..600.0));
    });

    let_effect!(|| {
        if let Some(window) = &*win.window().get($) {
            println!("window loaded {:?}", window);
        }
    });

    win.show(|ui| async move {
        join!(
            block(BlockProp { x, y }).show(ui),
            block(BlockProp { x, y }).show(ui)
        );
    })
    .await;
}

pub struct BlockProp<'a> {
    x: Pin<&'a StateCell<f64>>,
    y: Pin<&'a StateCell<f64>>,
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
                    element.x.set(prop.x.get($));
                });

                let_effect!(|| {
                    element.y.set(prop.y.get($));
                });

                child.show(ui).await;
                pending::<()>().await;
            },
        )
    })
}

#[derive(Debug)]
pub struct Block {
    pub x: Cell<f64>,
    pub y: Cell<f64>,
}

impl Block {
    pub fn new() -> Self {
        Self {
            x: Cell::new(0.0),
            y: Cell::new(0.0),
        }
    }
}

impl Element for Block {
    fn draw(self: Pin<&Self>, canvas: &Canvas, width: f32, height: f32) {
        let mut paint = Paint::new(Color4f::from(Color::GREEN), None);
        paint.set_style(PaintStyle::Fill);

        canvas.draw_rect(Rect::new(0.0,0.0, width, height), &paint);
    }
}
