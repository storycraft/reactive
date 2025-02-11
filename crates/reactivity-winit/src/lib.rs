pub mod event_loop;
pub mod resource;
pub mod state;

pub use async_executor::Task;
pub use event_loop::run;
pub use winit;

use core::future::Future;
use event_loop::context::AppCx;

pub fn spawn<Fut>(fut: Fut) -> Task<Fut::Output>
where
    Fut: Future + 'static,
{
    AppCx::with(|cx| cx.executor().spawn(fut))
}
