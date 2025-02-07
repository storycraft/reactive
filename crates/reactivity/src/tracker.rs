use core::pin::Pin;

use pin_project::pin_project;

use crate::{
    binding::{Binding, TrackerBinding},
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

    pub fn notify(self: Pin<&Self>) {
        Queue::with(|queue| {
            self.project_ref().dependents.take(|dependents| {
                for entry in dependents.iter() {
                    let entry = entry.value_pinned().to_handle();
                    if !entry.linked() {
                        continue;
                    }

                    let entry = unsafe { entry.value().get().as_ref() };
                    if !entry.linked() {
                        queue.add(entry);
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
