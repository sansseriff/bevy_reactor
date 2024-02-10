# TODO

* BUG: Dialog always creates content nodes even when closed.
* BUG: Dialog doesn't render a second time.
* BUG: "DisplayNodeChanged" warning.
* BUG: Can't render Switch or For as root of a presenter?
* Memo Signals
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

## Notes on fine-grained

#[derive(Component)]
struct Memo {
    value: Box<dyn Any>,
    deps_changed: bool,
}

create_memo:
    creates a tracking entity
    sets the current context to that entity
    attaches a "mark changed" action as the closure's action
    attaches the closure as the "recompute"
    runs the code
    restores the context
