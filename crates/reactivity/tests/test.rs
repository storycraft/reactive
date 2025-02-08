// use core::{
//     future::{poll_fn, Future},
//     pin::{pin, Pin},
// };

// use reactivity::{let_effect, queue::Queue, tracker::DependencyTracker};

// fn run_system<'a, Fut: Future + 'a>(f: impl FnOnce(Pin<&'a Queue>) -> Fut) -> Fut::Output {
//     let mut queue = pin!(Queue::new());
//     let mut fut = pin!(f(queue.as_ref()));

//     pollster::block_on(poll_fn(|cx| queue.(fut.as_mut(), cx)))
// }

// #[test]
// fn effects() {
//     run_system(async {
//         let a = pin!(DependencyTracker::new());
//         let a = a.into_ref();
//         let b = pin!(StateCell::new(0));
//         let b = b.into_ref();

//         let_effect!(|| {
//             b.set(a.get($) + 1);
//         });

//         a.update(|a| a + 1);
//     });
// }
