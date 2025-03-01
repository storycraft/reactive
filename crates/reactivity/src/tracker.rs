use core::pin::Pin;

use hkt_pin_list::LinkedList;
use pin_project::pin_project;

use crate::{
    effect::{Binding, TrackerBinding},
    queue::Queue,
};

#[pin_project]
#[derive(Debug)]
pub struct DependencyTracker {
    #[pin]
    dependents: LinkedList!(TrackerBinding),
}

impl DependencyTracker {
    pub fn new() -> Self {
        Self {
            dependents: LinkedList::new(),
        }
    }

    pub fn register(self: Pin<&Self>, binding: Binding) {
        self.project_ref()
            .dependents
            .push_front(binding.to_tracker());
    }

    pub fn notify(self: Pin<&Self>, queue: Pin<&Queue>) {
        self.project_ref().dependents.take(|dependents| {
            dependents.iter(|iter| {
                for dependent in iter {
                    let queue_node =
                        unsafe { Pin::new_unchecked(&*dependent.value_pinned().get()) };

                    if !queue_node.linked() {
                        queue.add(queue_node);
                    }
                }
            });
        });
    }
}

impl Default for DependencyTracker {
    fn default() -> Self {
        Self::new()
    }
}
