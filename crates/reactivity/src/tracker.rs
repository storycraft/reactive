use core::pin::Pin;

use pin_project::pin_project;

use crate::{
    effect::{Binding, TrackerBinding},
    list::List,
    queue::Queue,
};

#[pin_project]
#[derive(Debug)]
pub struct DependencyTracker {
    #[pin]
    dependents: List<TrackerBinding>,
}

impl DependencyTracker {
    pub fn new() -> Self {
        Self {
            dependents: List::new(),
        }
    }

    pub fn register(self: Pin<&Self>, binding: Pin<&Binding>) {
        self.project_ref()
            .dependents
            .push_front(binding.to_tracker());
    }

    pub fn notify(self: Pin<&Self>, queue: Pin<&Queue>) {
        self.project_ref().dependents.take(|dependents| {
            for dependent in dependents.iter() {
                let queue_entry = dependent.value_pinned().get_ref().get();

                if !queue_entry.linked() {
                    queue.add(queue_entry);
                }
            }
        });
    }
}

impl Default for DependencyTracker {
    fn default() -> Self {
        Self::new()
    }
}
