use core::{cell::Cell, pin::Pin, task::Waker};

use async_executor::LocalExecutor;
use hkt_pin_list::define_hkt_list;
use pin_project::pin_project;
use reactivity::queue::Queue;
use scoped_tls_hkt::scoped_thread_local;
use winit::event_loop::ActiveEventLoop;
use super::handler::WinitWindow;

scoped_thread_local!(static CX: for<'a> Context<'a>);

define_hkt_list!(pub HandlerList = for<'a, 'b> Pin<&'a (dyn WinitWindow + 'b)>);

#[derive(Clone, Copy)]
#[non_exhaustive]
pub struct Context<'a> {
    pub app: Pin<&'a AppShared>,
    pub el: &'a ActiveEventLoop,
}

#[pin_project]
pub struct AppShared {
    status: Cell<EventLoopStatus>,
    executor: LocalExecutor<'static>,
    #[pin]
    handlers: HandlerList,
    #[pin]
    queue: Queue,
}

impl AppShared {
    pub fn new(waker: Option<Waker>) -> Self {
        Self {
            status: Cell::new(EventLoopStatus::Suspended),
            executor: LocalExecutor::new(),
            handlers: HandlerList::new(),
            queue: Queue::new(waker),
        }
    }

    pub fn status(&self) -> EventLoopStatus {
        self.status.get()
    }

    pub(super) fn set_status(&self, status: EventLoopStatus) {
        self.status.set(status);
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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventLoopStatus {
    Resumed,
    Suspended,
}

pub fn set<R>(app: Pin<&AppShared>, el: &ActiveEventLoop, f: impl FnOnce() -> R) -> R {
    CX.set(Context { app, el }, f)
}

pub fn is_set() -> bool {
    CX.is_set()
}

pub fn with<R>(f: impl FnOnce(Context) -> R) -> R {
    CX.with(f)
}
