use core::{pin::Pin, ptr::NonNull};

use pin_project::pin_project;

use crate::{
    effect::{
        binding::{Binding, HandleBinding},
        handle::{with_handle, HandleEntryPtr},
    },
    list::List,
    Queue,
};

#[pin_project]
pub struct DependencyTracker {
    #[pin]
    dependents: List<HandleBinding>,
}

impl DependencyTracker {
    pub fn new() -> Self {
        Self {
            dependents: List::new(),
        }
    }

    pub fn register(self: Pin<&Self>, binding: Pin<&Binding>) {
        with_handle(move |handle| {
            let handle_entry = binding.handle_entry();
            handle_entry
                .value()
                .set(HandleEntryPtr::new(Some(NonNull::from(handle))));
            handle.value_pinned().list().push_front(handle_entry);

            self.project_ref()
                .dependents
                .push_front(binding.binding_entry());
        });
    }

    pub fn notify(self: Pin<&Self>) {
        Queue::with(|queue| {
            self.project_ref().dependents.take(|dependents| {
                for entry in dependents.iter() {
                    let entry = entry.value_pinned().handle_entry();
                    if !entry.linked() {
                        continue;
                    }

                    let Some(ptr) = *entry.value().take() else {
                        continue;
                    };

                    let entry = unsafe { ptr.as_ref() };
                    if !entry.linked() {
                        queue.add(entry);
                    }
                }
            });
        });
    }
}
