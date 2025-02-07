# Tbd
Experimental zero cost fine grained reactivity system

* `crates/example-app`: Minimal example of winit + skia with the reactivity system
* `crates/reactivity-macro`: Boilerplate proc-macro
* `crates/main`: reactivity system integration to winit
* `crates/reactivity`: Reactivity system implementation

Not working on no_std due to thread_local usage (fallback on single threaded environment is not implemented yet)