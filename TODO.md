# Things we need to demonstrate:

* Verify Razing / Despawning doesn't leak
* Reactive hover hook
* Reactive focus hook

# Other stuff

* No-arg .bind().
* Derived Signals
* Memo
* Effect
* Cleanup Handlers
* Scoped values
* Access to ECS components
* Access to owner entity
* use_element_rect hook
* transition hooks

# Obsidian

* Button
* Slider
* Splitter
* Flex
* etc...

## Notes on fine-grained

Tracking contexts. Start with create_memo:

    -- creates a tracking scope that is owned by the current context, like Solid
    -- it's a map of entity, component like before. Or resource.
    -- is it an entity?

#[derive(Component)]
struct Memo {
    value: Box<dyn Any>,
    deps_changed: bool,
}

#[derive(Component)]
struct Effect {
    action: fn()
}

create_memo:
    creates a tracking entity
    sets the current context to that entity
    attaches a "mark changed" action as the closure's action
    attaches the closure as the "recompute"
    runs the code
    restores the context

create_effect:
    creates a tracking entity
    sets the current context to that entity
    attaches the closure as that entity's action
    runs the code
    restores the context

# Hover:

  Inputs: element id - which needs create entity.

HoveringReaction
