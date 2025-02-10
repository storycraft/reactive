use core::{pin::Pin, task::Waker};
use std::rc::Rc;

use async_executor::LocalExecutor;
use pin_project::pin_project;
use reactivity::{define_safe_list, queue::Queue};
use scoped_tls_hkt::scoped_thread_local;

use super::handler::EventHandler;

scoped_thread_local!(static CX: Pin<Rc<AppCx>>);

define_safe_list!(pub HandlerList = Pin<&dyn EventHandler>);

#[pin_project]
pub struct AppCx {
    executor: LocalExecutor<'static>,
    #[pin]
    handlers: HandlerList,
    #[pin]
    queue: Queue,
}

impl AppCx {
    pub fn new(waker: Option<Waker>) -> Self {
        Self {
            executor: LocalExecutor::new(),
            handlers: HandlerList::new(),
            queue: Queue::new(waker),
        }
    }

    pub fn executor(&self) -> &LocalExecutor<'static> {
        &self.executor
    }

    pub fn handlers(self: Pin<&Self>) -> Pin<&HandlerList> {
        self.project_ref().handlers
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
