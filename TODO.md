# TODO

- Move 'inspector' widgets from obsidian_ui to obsidian_ui_inspect
- Menus:
  - Close on click
  - Keyboard navigation
  - Focus indicator
  - Restore Focus
  - Shortcuts
  - Checkmarks and checkmark spacing
- Property Editor.
  - Group:
    - Disclosure
    - Icon
    - Title
    - Dropdown
  - Undefined Fields
- Memoized Deriveds, with custom equality hook.
  - Needed to make Dynamic work with sliders
- Components and Resources as signals?
  - This is problematic because it would require Signal<T> to impl Component/Resource.
- Change tab key handling to use bubbled events.
- Restore focus, focus-visible when dialog closes.
- Clear focus when clicking on empty space.
- Verify Razing / Despawning doesn't leak
- use_element_rect hook - needed for popup menus
- Composite buffers.
- Don't execute dialog content if dialog not open.
- Can we make ForEach not require cloning the iterator?
- Text Input

  - drag to select (requires text measurement)
  - correct rendering of selection rects and cursor
  - correct rendering of focus rect (just uses outline for now)
  - correct rendering of rounded corners

# Node Graph

- split into its own crate
- color editor
- bug in shader when quadratics are straight
- input should be a polyline
- line colors should match terminal colors
- better shadows
- connecting and disconnecting
- line should appear above when dragging, and have rounded ends.
  - gesture

# StyleBuilder

- cursors

# Obsidian

- DisclosureTriangle - with transition
- Menu
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

- Figure out how to decouple Views from generic reactions. This would allow the low-level signal
  code to live in a separate crate.
- Currently For::each() requires cloning the iterator, but it shouldn't need to since only the
  closure is long-lived, not the iterator itself. Is there some way I can use Rust lifetimes to
  reduce the amount of cloning?
- A long-standing request is the ability to use Bevy queries, but I have never figured out how
  to do change detection on a query (that is, you can write a query that detects changes to
  components, but there's no way to detect a change to query results).

# Reflection Notes

- Starts with a Reflect object.
- For the root object, we treat container types (structs, lists, tuples) transparently, meaning
  that the contents of the inspector are the contents of the container. For value types,
  the contents of the container are the type itself.
- We need some way to register factories that can inspect the Reflect trait object and decide
  which widget to instantiate.
- Ultimately, this needs to return a ViewRef, or possibly Some(ViewRef).
- We also need to track which items are Optional/None and add them to the undefined fields list.
- We also need to sort the field names (maybe this should be a preference).
- Nested structs:
  - remove an optional item
  - add an optional item with a default value.
  - notify container that its contents have changed.
- Each widget has a path which allows access to the specific field. Yay!
