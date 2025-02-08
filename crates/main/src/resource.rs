use core::{future::Future, pin::Pin};
use std::rc::Rc;

use reactivity::binding::Binding;

use crate::{event_loop::context::AppCx, state::StateCell};

pub struct Resource<T> {
    status: Pin<Rc<StateCell<Option<T>>>>,
}

impl<T> Resource<T> {
    pub fn of<Fut>(f: impl FnOnce() -> Fut) -> Self
    where
        Fut: Future<Output = T> + 'static,
        T: 'static,
    {
        let fut = f();
        let status = Rc::pin(StateCell::new(None));

        if AppCx::is_set() {
            AppCx::with(|cx| {
                cx.executor()
                    .spawn({
                        let status = status.clone();

                        async move {
                            status.as_ref().set(Some(fut.await));
                        }
                    })
                    .detach();
            });
        }

        Self { status }
    }

    pub fn get(&self, binding: Pin<&Binding>) -> Option<T> {
        self.status.as_ref().take_get(binding)
    }
}
