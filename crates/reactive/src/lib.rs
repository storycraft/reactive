pub(crate) mod tree;
pub mod window;

use scopeguard::defer;
pub use taffy;

use core::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};
use pin_project::{pin_project, pinned_drop};
use reactivity_winit::winit::{event::WindowEvent, event_loop::ActiveEventLoop};
use skia_safe::Canvas;
use std::rc::Rc;
use taffy::{NodeId, Style};
use window::ui::Ui;

pub trait SetupFn<'a>
where
    Self: 'a,
{
    type Output;

    fn show(self, ui: Ui<'a>) -> impl Future<Output = Self::Output> + 'a;
}

impl<'a, F, Fut> SetupFn<'a> for F
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

#[pin_project(PinnedDrop)]
pub struct ElementFuture<'a, Fut> {
    ui: Ui<'a>,
    id: ElementId,
    #[pin]
    fut: Fut,
}

impl<Fut: Future> Future for ElementFuture<'_, Fut> {
    type Output = Fut::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.project().fut.poll(cx)
    }
}

#[pinned_drop]
impl<Fut> PinnedDrop for ElementFuture<'_, Fut> {
    fn drop(self: Pin<&mut Self>) {
        self.ui.remove(self.id);
    }
}

pub fn wrap_element<'a, T, Fut>(
    default_layout: Style,
    element: T,
    f: impl FnOnce(Ui<'a>, Pin<Rc<T>>) -> Fut + 'a,
) -> impl SetupFn<'a>
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
