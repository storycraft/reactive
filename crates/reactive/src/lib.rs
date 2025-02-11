pub(crate) mod tree;
pub mod window;

pub use taffy;

use core::{future::Future, pin::Pin};
use reactivity_winit::winit::{event::WindowEvent, event_loop::ActiveEventLoop};
use scopeguard::defer;
use skia_safe::Canvas;
use std::rc::Rc;
use taffy::{AvailableSpace, Layout, NodeId, Size, Style};
use window::ui::Ui;

/// Representation of a functional component.
///
/// This trait is implemented for all `FnOnce(Ui<'a>) -> impl Future + 'a` types.
pub trait SetupFn<'a>
where
    Self: 'a,
{
    type Output;

    fn show(self, ui: Ui<'a>) -> impl Future<Output = Self::Output> + 'a;
}

// For function components without children
impl<'a, F, Fut> SetupFn<'a> for F
where
    F: FnOnce(Ui<'a>) -> Fut + 'a,
    Fut: Future + 'a,
{
    type Output = Fut::Output;

    fn show(self, ui: Ui<'a>) -> impl Future<Output = Self::Output> + 'a {
        self(ui)
    }
}

impl<'a> SetupFn<'a> for () {
    type Output = ();

    async fn show(self, _: Ui<'a>) {}
}

/// Representation of a functional component with a child
pub trait SetupFnWithChild<'a, Child> {
    type Output;

    fn child(self, child: Child) -> impl SetupFn<'a, Output = Self::Output>;
}

impl<'a, F, Child, Fut> SetupFnWithChild<'a, Child> for F
where
    F: FnOnce(Ui<'a>, Child) -> Fut + 'a,
    Child: SetupFn<'a>,
    Fut: Future + 'a,
{
    type Output = Fut::Output;

    fn child(self, child: Child) -> impl SetupFn<'a, Output = Fut::Output> {
        |ui| self(ui, child)
    }
}

#[easy_ext::ext(SetupFnWithChildExt)]
pub impl<'a, F> F
where
    F: SetupFnWithChild<'a, ()>,
{
    fn show(self, ui: Ui<'a>) -> impl Future<Output = F::Output> + 'a {
        self.child(()).show(ui)
    }
}

/// Wrap functional component with a child
pub fn with_children<'a, Child, C>(
    f: impl FnOnce(Child) -> C + 'a,
) -> impl SetupFnWithChild<'a, Child>
where
    C: SetupFn<'a> + 'a,
    Child: SetupFn<'a>,
{
    |ui, child| f(child).show(ui)
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
#[repr(transparent)]
pub struct ElementId(NodeId);

pub trait Element: 'static {
    fn on_event(self: Pin<&Self>, _el: &ActiveEventLoop, _event: &mut WindowEvent) {}

    // Draw element
    fn draw(self: Pin<&Self>, _canvas: &Canvas, _layout: &Layout) {}

    // Measure content size of element
    fn measure(
        self: Pin<&Self>,
        _known_dimensions: Size<Option<f32>>,
        _available_space: Size<AvailableSpace>,
        _style: &Style,
    ) -> Size<f32> {
        Size::ZERO
    }

    // Called before drawing relative positioned children
    fn pre_child_draw(self: Pin<&Self>, _canvas: &Canvas) {}
    // Called after drawing relative positioned children
    fn post_child_draw(self: Pin<&Self>, _canvas: &Canvas) {}
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
