use core::pin::pin;

use reactive::event::{EventListener, EventTarget};

fn main() {
    let target = pin!(EventTarget::<i32>::new());
    let target = target.into_ref();

    let mut i = 0;

    let f = &mut |i: &mut _| {
        *i += 10;
        false
    };

    let listener = pin!(EventListener::new(f));
    target.on(listener.into_ref());
    {
        let listener = pin!(EventListener::new(f));
        target.on(listener.into_ref());

        target.emit(&mut i);

        dbg!(i);
    }

    target.emit(&mut i);
    dbg!(i);
}
