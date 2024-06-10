# TODO

- Checkbox bug in inspector
  - caused by run reactions?
  - Need "run to convergence".
- Menus:
  - Restore Focus
  - Shortcuts
  - Checkmarks and checkmark spacing
- Property Editor.
  - Tuples (Nested)
  - Map
  - Array
  - Top level tuples
  - Top level newtype structs
  - Top level lists
  - Spinbox click to edit text
  - Slider click to edit text
- Too many public members in TrackingScope. This happened because of the need to
  allow external crates to define their own reaction types.
- `run_reactions` does a lot of excessive entity lookups, needed to satisfy the borrow checker.
- Components and Resources as signals?
  - This is problematic because it would require Signal<T> to impl Component/Resource.
- Change tab key handling to use bubbled events.
- Restore focus, focus-visible when dialog closes.
- Clear focus when clicking on empty space.
- Verify Razing / Despawning doesn't leak
- Composite buffers.
- Don't execute dialog content if dialog not open. (Maybe this already happens? Not sure.)
- Can we make ForEach not require cloning the iterator?
- Persist recent colors
- Idea: dynamic style effects.
  - Nice, but borrowing...
- Text Input
  - drag to select (requires text measurement)
  - correct rendering of selection rects and cursor
  - correct rendering of focus rect (just uses outline for now)
  - correct rendering of rounded corners

# Node Graph

- split into its own crate
- color editor (embedded graph node version)
- bug in shader when quadratics are straight
- input should be a polyline
- line colors should match terminal colors
- better shadows
- connecting and disconnecting
- line should appear above when dragging, and have rounded ends.
  - gesture

# Obsidian

- Focus Outlines (improve appearance)
- Graph editor.

# Possible crate structure

- bevy_reactor_signals
- bevy_reactor_views
- bevy_reactor_styles

- obsidian_ui_core
- obsidian_ui_controls
- obsidian_ui_reflect
- obsidian_ui_graph

# Sticking Points

- Currently For::each() requires cloning the iterator, but it shouldn't need to since only the
  closure is long-lived, not the iterator itself. Is there some way I can use Rust lifetimes to
  reduce the amount of cloning?
- A long-standing request is the ability to use Bevy queries, but I have never figured out how
  to do change detection on a query (that is, you can write a query that detects changes to
  components, but there's no way to detect a change to query results).
