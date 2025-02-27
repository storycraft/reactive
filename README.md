# Tbd
Experimental zero cost declarative fine grained reactivity system

## How it works
![structure](assets/structure.svg)
 * `DependencyTracker`: 1 pointer sized main reactive primitive wrapping Intrusive linked list. Which contains start pointer to `Binding`
 * `Binding`: Node of `DependencyTracker`. Unlinked on drop. Each node contains pointer to `Effect`'s queue node
 * `Effect`: Contains constant number of bindings(Each one for depending `DependencyTracker`) and a node which can be linked to a queue
 * `Queue`: Lazily evaluated effect queue. Evaluate queued effects and provides an asynchronous interface.

1. `DependencyTracker`s are created and pinned
2. `Effect`s are created, pinned, initialized and evaluate the effect closure once
3. During evaluation, each `Binding`s are attached to a corresponding `DependencyTracker`
4. When a value changes, corresponding `DependencyTracker` is notified.
5. Notified `DependencyTracker` traverses linked `Binding` nodes, link the `Effect` node to the `Queue` and clear the list
6. `Waker` in the `Queue` notify upper `Executor` when nodes are added
7. On next poll, the `Queue` pops first linked node, run effect closure. Repeat until there is no nodes left in the queue

See `crates/example-app` for more complex example

> [!NOTE]
> Number of required bindings are calculated in compile time(using macro).

## License
The project is dual-licensed under Apache-2.0 and MIT.