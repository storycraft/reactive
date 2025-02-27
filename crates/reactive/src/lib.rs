pub(crate) mod tree;
pub mod window;
mod macros;

pub use reactivity_winit;
pub use reactivity_winit::winit;
pub use skia_safe;
pub use taffy;

use core::{
    any::{Any, TypeId},
    future::{pending, Future},
    pin::Pin,
};
use reactivity_winit::winit::{event::WindowEvent, event_loop::ActiveEventLoop};
use scopeguard::defer;
use skia_safe::Canvas;
use taffy::{AvailableSpace, Layout, NodeId, Size, Style};
use window::ui::Ui;

/// Representation of a functional component.
///
/// This trait is implemented for all `FnOnce(Ui<'a>) -> impl Future + 'a` types.
pub trait SetupFn {
    type Output;

    fn show(self, ui: Ui) -> impl Future<Output = Self::Output>;
}

// For function components without children
impl<F, Fut> SetupFn for F
where
    F: FnOnce(Ui) -> Fut,
    Fut: Future,
{
    type Output = Fut::Output;

    fn show(self, ui: Ui) -> impl Future<Output = Self::Output> {
        self(ui)
    }
}

impl SetupFn for () {
    type Output = ();

    async fn show(self, _: Ui) {
        pending::<()>().await;
    }
}

/// Representation of a functional component with a child
pub trait WithChild<Child> {
    type Output;

    fn child(self, child: Child) -> impl SetupFn<Output = Self::Output>;
}

impl<F, Child, Fut> WithChild<Child> for F
where
    F: FnOnce(Ui, Child) -> Fut,
    Fut: Future,
{
    type Output = Fut::Output;

    fn child(self, child: Child) -> impl SetupFn<Output = Fut::Output> {
        |ui| self(ui, child)
    }
}

#[easy_ext::ext(SetupFnWithChildExt)]
pub impl<F> F
where
    F: WithChild<()>,
{
    fn show(self, ui: Ui) -> impl Future<Output = F::Output> {
        self.child(()).show(ui)
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
#[repr(transparent)]
pub struct ElementId(NodeId);

/// Smallest draw unit with a layout
pub trait Element: Any {
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

impl dyn Element {
    pub(crate) fn downcast_ref<T: Element>(self: Pin<&Self>) -> Option<Pin<&T>> {
        let tid = Any::type_id(self.get_ref());

        if tid == TypeId::of::<T>() {
            Some(unsafe { self.map_unchecked(move |el| &*(el as *const dyn Element as *const T)) })
        } else {
            None
        }
    }

    pub(crate) fn downcast_mut<T: Element>(self: Pin<&mut Self>) -> Option<Pin<&mut T>> {
        let tid = Any::type_id(self.as_ref().get_ref());

        if tid == TypeId::of::<T>() {
            Some(unsafe {
                self.map_unchecked_mut(move |el| &mut *(el as *mut dyn Element as *mut T))
            })
        } else {
            None
        }
    }
}

pub fn create_element<F: SetupFn>(
    element: impl Element,
    default_layout: Style,
    f: F,
) -> impl SetupFn<Output = F::Output> {
    |ui: Ui| async move {
        let id = ui.append(default_layout, element);
        defer!(ui.remove(id));

        f.show(ui.sub_ui(id)).await
    }
}
