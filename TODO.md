# TODO

- Refactor Mutables (generic)
- Resource timestamps.
- Reaction cleanups
- Scrollbar
- Scroll view
- Scoped values
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
