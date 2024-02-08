# TODO

* Memo Signals
* create_effect()
* Verify Razing / Despawning doesn't leak
* Reactive focus hook
* No-arg .bind().
* Cleanup Handlers - on_cleanup();
* Scoped values
* Access to ECS components
* Access to owner entity
* use_element_rect hook
* enter/exit transition hooks
* Animated transitions / spring
* Gizmo

# StyleBuilder

* cursors
* line break

# Obsidian

* Checkbox (finish rounded corners)
* Radio
* Button Group
* DisclosureTriangle - with transition
* Flex
* Multi-layered nine-patch button.
* Modal
* Menu
* Focus Outlines
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
