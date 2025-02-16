use core::pin::Pin;

use pin_project::pin_project;

use crate::{
    effect::{Binding, TrackerBinding},
    queue::Queue,
};
use hkt_pin_list::define_safe_list;

define_safe_list!(pub(crate) TrackerList = TrackerBinding);

#[pin_project]
#[derive(Debug)]
pub struct DependencyTracker {
    #[pin]
    dependents: TrackerList,
}

impl DependencyTracker {
    pub fn new() -> Self {
        Self {
            dependents: TrackerList::new(),
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
                    let queue_entry = unsafe { dependent.value_pinned().get().as_ref() };

                    if !queue_entry.linked() {
                        queue.add(queue_entry);
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
