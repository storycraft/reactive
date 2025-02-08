use core::pin::Pin;
use std::rc::Rc;

use async_executor::LocalExecutor;
use pin_project::pin_project;
use reactivity::{list::List, queue::Queue};
use scoped_tls_hkt::scoped_thread_local;

use super::components::ComponentKey;

scoped_thread_local!(static CX: Pin<Rc<AppCx>>);

#[pin_project]
pub struct AppCx {
    executor: LocalExecutor<'static>,
    #[pin]
    components: List<ComponentKey>,
    #[pin]
    queue: Queue,
}

impl AppCx {
    pub fn new() -> Self {
        Self {
            executor: LocalExecutor::new(),
            components: List::new(),
            queue: Queue::new(),
        }
    }

    pub fn executor(&self) -> &LocalExecutor<'static> {
        &self.executor
    }

    pub fn components(self: Pin<&Self>) -> Pin<&List<ComponentKey>> {
        self.project_ref().components
    }

    pub fn queue(self: Pin<&Self>) -> Pin<&Queue> {
        self.project_ref().queue
    }

    pub fn set<R>(self: &Pin<Rc<AppCx>>, f: impl FnOnce() -> R) -> R {
        CX.set(self, f)
    }

    pub fn is_set() -> bool {
        CX.is_set()
    }

    pub fn with<R>(f: impl FnOnce(&Pin<Rc<AppCx>>) -> R) -> R {
        CX.with(f)
    }
}
