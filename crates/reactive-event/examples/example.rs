use core::pin::pin;

use reactive_event::{EventTarget, Listener};

fn main() {
    let target = pin!(<EventTarget!(&mut i32)>::new());
    let target = target.into_ref();

    {
        let mut b = 2;
        let listener = pin!(Listener::new(|a: &mut i32| {
            dbg!(*a + b);
            *a += 1;
            b += 2;
            false
        }));
        target.bind(listener);

        let listener2 = pin!(Listener::new(|a: &mut i32| {
            dbg!(*a + 5);
            true
        }));
        target.bind(listener2);

        target.emit_mut(&mut 2);
    }

    // nothing happens because all listeners are dropped
    target.emit_mut(&mut 0);
}
