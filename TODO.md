# TODO

* Text input.
* Change tab handling to use bubbled events.
* Dialogs should focus.
* ESC to close dialog.
* Rounded corners for sliders.
* Memoized Signals
* Verify Razing / Despawning doesn't leak
* No-arg .bind().
* Cleanup Handlers - on_cleanup();
* Scoped values
* Access to owner entity
* use_element_rect hook - needed for popup menus
* Gizmo
* Composite buffers.

# StyleBuilder

* cursors
* line break

# Obsidian

* Checkbox (finish rounded corners)
* Radio
* DisclosureTriangle - with transition
* Multi-layered nine-patch button.
* Menu
* Focus Outlines (improve appearance)
* Modal animation.

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
