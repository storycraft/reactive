pub mod event_loop;
pub mod resource;
pub mod state;

pub use async_executor::Task;
pub use event_loop::run;
pub use winit;

use core::future::Future;
use event_loop::context;

// TODO:: multi threaded
pub fn spawn_ui<Fut>(fut: Fut) -> Task<Fut::Output>
where
    Fut: Future + 'static,
{
    context::with(|cx| cx.app.executor().spawn(fut))
}
