use core::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use never_say_never::Never;

#[derive(Debug, Clone, Copy)]
pub struct NoChild;

impl Future for NoChild {
    type Output = Never;

    fn poll(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Self::Output> {
        Poll::Pending
    }
}
