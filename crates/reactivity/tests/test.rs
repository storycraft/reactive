use core::{
    future::{poll_fn, Future},
    pin::pin,
};

use reactivity::{let_effect, queue::Queue, state::StateCell};

fn run_system<F: Future>(fut: F) -> F::Output {
    let mut queue = pin!(Queue::new());
    let mut fut = pin!(fut);

    pollster::block_on(poll_fn(|cx| queue.as_mut().poll(fut.as_mut(), cx)))
}

#[test]
fn effects() {
    run_system(async {
        let a = pin!(StateCell::new(0));
        let a = a.into_ref();
        let b = pin!(StateCell::new(0));
        let b = b.into_ref();

        let_effect!(|| {
            b.set(a.get($) + 1);
        });
        
        a.update(|a| a + 1);
    });
}
