pub(crate) mod tree;
pub mod window;

pub use taffy;

use core::{future::Future, pin::Pin};
use reactivity_winit::winit::{event::WindowEvent, event_loop::ActiveEventLoop};
use scopeguard::defer;
use skia_safe::Canvas;
use std::rc::Rc;
use taffy::{NodeId, Style};
use window::ui::Ui;

/// Representation of a component.
/// 
/// This trait is implemented for all `FnOnce(Ui<'a>) -> impl Future + 'a` types.
pub trait Component<'a>
where
    Self: 'a,
{
    type Output;

    fn show(self, ui: Ui<'a>) -> impl Future<Output = Self::Output> + 'a;
}

impl<'a, F, Fut> Component<'a> for F
where
    F: FnOnce(Ui<'a>) -> Fut + 'a,
    Fut: Future + 'a,
{
    type Output = Fut::Output;

    fn show(self, ui: Ui<'a>) -> impl Future<Output = Self::Output> + 'a {
        (self)(ui)
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
#[repr(transparent)]
pub struct ElementId(NodeId);

pub trait Element: 'static {
    fn on_event(self: Pin<&Self>, _el: &ActiveEventLoop, _event: &mut WindowEvent) {}

    // Draw elements
    fn draw(self: Pin<&Self>, _canvas: &Canvas) {}

    // Called before drawing relative positioned children
    fn pre_child_draw(self: Pin<&Self>, _canvas: &Canvas) {}
    // Called after drawing relative positioned children
    fn post_child_draw(self: Pin<&Self>, _canvas: &Canvas) {}
}

pub fn wrap_element<'a, T, Fut>(
    default_layout: Style,
    element: T,
    f: impl FnOnce(Ui<'a>, Pin<Rc<T>>) -> Fut + 'a,
) -> impl Component<'a>
where
    T: Element + 'static,
    Fut: Future + 'a,
{
    move |ui: Ui<'a>| async move {
        let (id, element) = ui.append(default_layout, element);
        defer!(ui.remove(id));

        (f)(Ui::new(ui.tree(), id), element).await
    }
}
