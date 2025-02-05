// // test_component
// component!(
//     pub TestComponent {
//         number: i32,
//     } -> {
//         a: number + 1,
//     }
// )

// // app
// component!(
//     pub App {} -> {
//         start: 0,
//         next: component.a,
//         component: TestComponent {
//             number: start,
//         },
//     }
// )

use core::pin::pin;

use async_ui_web::{
    event_traits::EmitElementEvent,
    html::{Button, Div, Paragraph},
    join, NoChild,
};
use reactive::{
    self,
    effect::{binding::Binding, Effect},
    state::StateCell,
};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn run() {
    async_ui_web::mount(reactive::run(app()));
}

async fn app() {
    let counter = pin!(StateCell::new(0));
    let counter = counter.as_ref();

    let text = Paragraph::new();
    let btn = Button::new();
    btn.set_inner_text("Click");

    let binding = pin!(Binding::new());
    let binding = binding.as_ref();
    let effect = pin!(Effect::new(|| {
        text.set_inner_text(&format!("Counter is updated to {}", counter.get(binding)));
    }));
    effect.init();

    Div::new()
        .render(join((text.render(NoChild), btn.render(NoChild), async {
            loop {
                btn.until_click().await;
                counter.set(counter.get_untracked() + 1);
            }
        })))
        .await;
}
