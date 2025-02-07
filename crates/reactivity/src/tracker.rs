use core::pin::Pin;

use pin_project::pin_project;

use crate::{
    binding::{Binding, TrackerBinding},
    effect::handle::EffectFnPtrExt,
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
                for dependent in dependents.iter() {
                    let handle_entry = dependent.value_pinned().to_handle();

                    if let Some(queue_entry) = handle_entry.to_queue() {
                        queue.add(queue_entry);
                    } else {
                        continue;
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
