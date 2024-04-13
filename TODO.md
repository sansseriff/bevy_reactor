# TODO

- Reaction cleanups
- Tracking scope cleanups.
- Scrollbar
- Scroll view
- Build a router that uses Bevy states.
- Change tab handling to use bubbled events.
- Restore focus, focus-visible when dialog closes.
- Clear focus when clicking on empty space.
- Memoized Signals
- Verify Razing / Despawning doesn't leak
- No-arg .bind().
- Cleanup Handlers - on_cleanup();
- use_element_rect hook - needed for popup menus
- Composite buffers.
- Text Input

  - drag to select (requires text measurement)
  - correct rendering of selection rects and cursor
  - correct rendering of focus rect (just uses outline for now)
  - correct rendering of rounded corners

- Scoped values:
  - In order to make this efficient, we need some way to represent a map as components,
    such that we can do change detection on a single key/value pair rather than the whole map.
  - Cx will need to add the owner entity as a property.
  - Every owner entity will need to be parented (most are already).
  - This will let us start the search at the current Cx and walk up the tree of ancestors.

# StyleBuilder

- cursors

# Obsidian

- Radio
- DisclosureTriangle - with transition
- Multi-layered nine-patch button.
- Menu
- Focus Outlines (improve appearance)
- Modal animation.

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
