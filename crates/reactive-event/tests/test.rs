use core::pin::pin;

use reactive_event::{EventTarget, Listener};

#[test]
fn test() {
    let target = pin!(<EventTarget!(&mut i32)>::new());
    let target = target.into_ref();

    {
        let mut b = 2;
        let listener = pin!(Listener::new(|a: &mut i32| {
            dbg!(*a + b);
            b += 2;
            false
        }));
        target.bind(listener.as_ref());

        let listener2 = pin!(Listener::new(|a: &mut i32| {
            dbg!(*a + 5);
            true
        }));
        target.bind(listener2);

        target.emit_mut(&mut 2);
    }
}
