# TODO

- Property Editor.
- Popup menus
- Memoized Deriveds, with custom equality hook.
  - Needed to make Dynamic work with sliders
- Components and Resources as signals?
  - This is problematic because it would require Signal<T> to impl Component/Resource.
- Change tab key handling to use bubbled events.
- Restore focus, focus-visible when dialog closes.
- Clear focus when clicking on empty space.
- Verify Razing / Despawning doesn't leak
- No-arg .bind().
  - Or possibly get rid of function presenters altogether.
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
