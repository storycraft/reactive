use core::{future::Future, pin::Pin};
use std::rc::Rc;

use reactivity::binding::Binding;

use crate::{event_loop::context::AppCx, state::StateCell};

pub struct Resource<T> {
    state: Pin<Rc<StateCell<Option<T>>>>,
}

impl<T> Resource<T> {
    pub fn new() -> Self {
        Self {
            state: Rc::pin(StateCell::new(None)),
        }
    }

    pub fn load<Fut>(&self, fut: Fut)
    where
        Fut: Future<Output = T> + 'static,
        T: 'static,
    {
        AppCx::with(|cx| {
            let state = self.state.clone();

            cx.executor()
                .spawn({
                    async move {
                        state.as_ref().set(Some(fut.await));
                    }
                })
                .detach();
        });
    }

    pub fn get(&self, binding: Pin<&Binding>) -> Option<T> {
        self.state.as_ref().take_get(binding)
    }
}
