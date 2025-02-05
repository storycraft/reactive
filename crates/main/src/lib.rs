#![cfg_attr(feature = "no-std", no_std)]

pub mod effect;
pub(crate) mod list;
pub mod state;
mod queue;
pub mod tracker;

use core::{future::Future, pin::Pin};

use queue::Queue;

pub trait Component {
    fn init(self: Pin<&mut Self>);
}

#[inline]
pub async fn run<F: Future>(app: F) -> F::Output {
    Queue::run(app).await
}
