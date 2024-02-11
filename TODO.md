# TODO

* Memo Signals
* Verify Razing / Despawning doesn't leak
* Reactive focus hook
* No-arg .bind().
* Cleanup Handlers - on_cleanup();
* Scoped values
* Access to owner entity
* use_element_rect hook - needed for popup menus
* Animated transitions / spring
* Gizmo
* Focus keys.
* ESC to close dialog.

# StyleBuilder

* cursors
* line break

# Obsidian

* Checkbox (finish rounded corners)
* Radio
* Button Group (Needed rounded corners first)
* DisclosureTriangle - with transition
* Flex
* Multi-layered nine-patch button.
* Menu
* Focus Outlines
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
