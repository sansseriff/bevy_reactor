use std::sync::Arc;

use bevy::{prelude::*, ui};
use bevy_mod_stylebuilder::*;
use bevy_reactor_builder::{
    CreateChilden, EntityStyleBuilder, InvokeUiTemplate, UiBuilder, UiTemplate,
};
use bevy_reactor_signals::Mutable;

use crate::scrolling::{ScrollArea, ScrollBar, ScrollBarThumb, ScrollContent, ScrollWheelEvent};

// Style definitions for scrollview widget.

// The combined scroll view with scrolling region and scrollbars.
fn style_scroll_view(ss: &mut StyleBuilder) {
    ss.display(ui::Display::Grid)
        .grid_template_columns(vec![
            ui::RepeatedGridTrack::flex(1, 1.),
            ui::RepeatedGridTrack::auto(1),
        ])
        .grid_template_rows(vec![
            ui::RepeatedGridTrack::flex(1, 1.),
            ui::RepeatedGridTrack::auto(1),
        ])
        .gap(2);
}

/// The scrolling region which defines the clipping bounds.
fn style_scroll_region(ss: &mut StyleBuilder) {
    ss.grid_column(ui::GridPlacement::start_span(1, 1))
        .grid_row(ui::GridPlacement::start_span(1, 1))
        .overflow(ui::OverflowAxis::Clip);
}

/// The scrolling content which is clipped.
fn style_scroll_content(ss: &mut StyleBuilder) {
    ss.position(ui::PositionType::Absolute)
        .height(ui::Val::Auto)
        .min_width(ui::Val::Percent(100.))
        .border(1);
}

fn style_scrollbar_x(ss: &mut StyleBuilder) {
    ss.grid_column(ui::GridPlacement::start_span(1, 1))
        .grid_row(ui::GridPlacement::start_span(2, 1))
        .height(8);
}

fn style_scrollbar_x_thumb(ss: &mut StyleBuilder) {
    ss.background_color("#334")
        .position(ui::PositionType::Absolute)
        .top(1)
        .bottom(1)
        .border_radius(3);
    // .selector(":hover > &,.drag", |ss| ss.background_color("#556"));
}

fn style_scrollbar_y(ss: &mut StyleBuilder) {
    ss.grid_column(ui::GridPlacement::start_span(2, 1))
        .grid_row(ui::GridPlacement::start_span(1, 1))
        .width(8);
}

fn style_scrollbar_y_thumb(ss: &mut StyleBuilder) {
    ss.background_color("#334")
        .position(ui::PositionType::Absolute)
        .left(1)
        .right(1)
        .border_radius(3);
    // .selector(":hover > &,.drag", |ss| ss.background_color("#556"));
}

#[derive(Clone, PartialEq, Default, Copy)]
enum DragMode {
    #[default]
    None,
    DragX,
    DragY,
}

#[derive(Clone, PartialEq, Default, Copy)]
struct DragState {
    mode: DragMode,
    offset: f32,
}

/// The scroll view widget.
pub struct ScrollView {
    /// Views for the scrolling content
    pub children: Arc<dyn Fn(&mut UiBuilder)>,
    /// Style to be applied to the entire scroll view,
    pub style: StyleHandle,
    /// Style to be applied to the content region,
    pub content_style: StyleHandle,
    /// Whether to enable horizontal scrolling.
    pub scroll_enable_x: bool,
    /// Whether to enable vertical scrolling.
    pub scroll_enable_y: bool,
    /// Optional entity id to use for the scrolling element. This is useful for querying the
    /// current scroll position.
    pub entity: Option<Entity>,
}

impl Default for ScrollView {
    fn default() -> Self {
        Self {
            children: Arc::new(|_| {}),
            style: Default::default(),
            content_style: Default::default(),
            scroll_enable_x: Default::default(),
            scroll_enable_y: Default::default(),
            entity: Default::default(),
        }
    }
}

impl ScrollView {
    /// Create a new `ScrollView`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the child views for this element.
    pub fn children<V: 'static + Fn(&mut UiBuilder)>(mut self, children: V) -> Self {
        self.children = Arc::new(children);
        self
    }

    /// Set additional styles to be applied to the scroll view.
    pub fn style<S: StyleTuple + 'static>(mut self, style: S) -> Self {
        self.style = style.into_handle();
        self
    }

    /// Set additional styles to be applied to the scroll content.
    pub fn content_style<S: StyleTuple + 'static>(mut self, style: S) -> Self {
        self.content_style = style.into_handle();
        self
    }

    /// Enable horizontal scrolling.
    pub fn scroll_enable_x(mut self, enable: bool) -> Self {
        self.scroll_enable_x = enable;
        self
    }

    /// Enable vertical scrolling.
    pub fn scroll_enable_y(mut self, enable: bool) -> Self {
        self.scroll_enable_y = enable;
        self
    }

    /// Set the entity id to use for the scrolling element.
    /// This is useful for querying the current scroll position.
    pub fn entity(mut self, entity: Entity) -> Self {
        self.entity = Some(entity);
        self
    }
}

impl UiTemplate for ScrollView {
    fn build(&self, builder: &mut UiBuilder) {
        // A widget which displays a scrolling view of its children.
        let enable_x = self.scroll_enable_x;
        let enable_y = self.scroll_enable_y;
        let drag_state = builder.create_mutable::<DragState>(DragState::default());

        if let Some(entity) = self.entity {
            let mut e = builder.entity_mut(entity);
            e.insert(Name::new("ScrollView"));
            e
        } else {
            builder.spawn((NodeBundle::default(), Name::new("ScrollView")))
        }
        .styles((style_scroll_view, self.style.clone()))
        .create_children(|builder| {
            // Scroll area
            let id_scroll_area = builder
                .spawn((NodeBundle::default(), Name::new("ScrollView::ScrollArea")))
                .id();

            let id_scrollbar_x = if enable_x {
                Some(builder.spawn_empty().id())
            } else {
                None
            };
            let id_scrollbar_y = if enable_y {
                Some(builder.spawn_empty().id())
            } else {
                None
            };

            builder
                .entity_mut(id_scroll_area)
                .insert((
                    ScrollArea {
                        id_scrollbar_x,
                        id_scrollbar_y,
                        ..default()
                    },
                    // On::<ScrollWheel>::listener_component_mut::<ScrollArea>(
                    //     move |ev, scrolling| {
                    //         ev.stop_propagation();
                    //         scrolling.scroll_by(-ev.delta.x, -ev.delta.y);
                    //     },
                    // ),
                ))
                .style(style_scroll_region)
                .create_children(|builder| {
                    builder
                        .spawn((NodeBundle::default(), Name::new("ScrollView::ScrollRegion")))
                        .insert(ScrollContent)
                        .styles((style_scroll_content, self.content_style.clone()))
                        .create_children(|builder| {
                            (self.children.as_ref())(builder);
                        });
                });

            // Horizontal scroll bar
            if let Some(id_scrollbar) = id_scrollbar_x {
                builder.invoke(Scrollbar::new(ScrollbarProps {
                    id_scroll_area,
                    id_scrollbar,
                    drag_state,
                    vertical: false,
                }));
            }

            if let Some(id_scrollbar) = id_scrollbar_y {
                builder.invoke(Scrollbar::new(ScrollbarProps {
                    id_scroll_area,
                    id_scrollbar,
                    drag_state,
                    vertical: true,
                }));
            }
        });
    }
}

/// Properties for the `Scrollbar` widget.
#[derive(Clone)]
pub struct ScrollbarProps {
    id_scroll_area: Entity,
    id_scrollbar: Entity,
    drag_state: Mutable<DragState>,
    vertical: bool,
}

/// Scrollbar widget.
pub struct Scrollbar(ScrollbarProps);

impl Scrollbar {
    /// Create a new `Scrollbar`.
    pub fn new(props: ScrollbarProps) -> Self {
        Self(props)
    }
}

impl UiTemplate for Scrollbar {
    fn build(&self, builder: &mut UiBuilder) {
        let vertical = self.0.vertical;
        let drag_state = self.0.drag_state;
        let id_scroll_area = self.0.id_scroll_area;
        let id_thumb = builder
            .spawn((NodeBundle::default(), Name::new("Scrollbar")))
            .id();
        builder
            .entity_mut(id_thumb)
            .insert((
                ScrollBar {
                    id_scroll_area,
                    vertical,
                    min_thumb_size: 10.,
                },
                // Click outside of thumb
                // On::<Pointer<DragStart>>::run(
                //     move |mut ev: ListenerMut<Pointer<DragStart>>,
                //           mut query: Query<&mut ScrollArea>,
                //           query_thumb: Query<(
                //         &Node,
                //         &mut ScrollBarThumb,
                //         &GlobalTransform,
                //     )>| {
                //         ev.stop_propagation();
                //         if let Ok(mut scroll_area) = query.get_mut(id_scroll_area) {
                //             if let Ok((thumb, _, transform)) = query_thumb.get(id_thumb) {
                //                 // Get thumb rectangle
                //                 let rect = thumb.logical_rect(transform);
                //                 handle_track_click(
                //                     &mut scroll_area,
                //                     vertical,
                //                     ev.pointer_location.position,
                //                     rect,
                //                 );
                //             }
                //         };
                //     },
                // ),
            ))
            .style(if vertical {
                style_scrollbar_y
            } else {
                style_scrollbar_x
            })
            .create_children(|builder| {
                builder
                    .entity_mut(id_thumb)
                    // .class_names(CLS_DRAG.if_true(cx.read_atom(drag_state).mode == mode))
                    .style(if vertical {
                        style_scrollbar_y_thumb
                    } else {
                        style_scrollbar_x_thumb
                    })
                    .insert((
                        ScrollBarThumb,
                        // Click/Drag on thumb
                        // On::<Pointer<DragStart>>::run(move |world: &mut World| {
                        //     let mut event = world
                        //         .get_resource_mut::<ListenerInput<Pointer<DragStart>>>()
                        //         .unwrap();
                        //     event.stop_propagation();
                        //     if let Some(scroll_area) = world.get::<ScrollArea>(id_scroll_area) {
                        //         drag_state.set(
                        //             world,
                        //             DragState {
                        //                 mode: DragMode::DragY,
                        //                 offset: if vertical {
                        //                     scroll_area.scroll_top
                        //                 } else {
                        //                     scroll_area.scroll_left
                        //                 },
                        //             },
                        //         );
                        //     }
                        // }),
                        // On::<Pointer<Drag>>::run(move |world: &mut World| {
                        //     let mut event = world
                        //         .get_resource_mut::<ListenerInput<Pointer<Drag>>>()
                        //         .unwrap();
                        //     event.stop_propagation();
                        //     let distance = event.distance;
                        //     let ds = drag_state.get(world);
                        //     if let Some(mut scroll_area) =
                        //         world.get_mut::<ScrollArea>(id_scroll_area)
                        //     {
                        //         handle_thumb_drag(&mut scroll_area, &ds, distance);
                        //     }
                        // }),
                        // On::<Pointer<DragEnd>>::run(move |world: &mut World| {
                        //     let mut event = world
                        //         .get_resource_mut::<ListenerInput<Pointer<DragEnd>>>()
                        //         .unwrap();
                        //     event.stop_propagation();
                        //     drag_state.set(
                        //         world,
                        //         DragState {
                        //             mode: DragMode::None,
                        //             offset: 0.,
                        //         },
                        //     );
                        // }),
                        // On::<Pointer<PointerCancel>>::run(
                        //     move |mut ev: ListenerMut<Pointer<DragEnd>>, mut atoms: AtomStore| {
                        //         ev.stop_propagation();
                        //         handle_thumb_drag_end(&mut atoms, drag_state);
                        //     },
                        // ),
                    ));
            });
    }
}

fn handle_thumb_drag(scroll_area: &mut ScrollArea, ds: &DragState, distance: Vec2) {
    if ds.mode == DragMode::DragY {
        let left = scroll_area.scroll_left;
        let top = if scroll_area.visible_size.y > 0. {
            ds.offset + distance.y * scroll_area.content_size.y / scroll_area.visible_size.y
        } else {
            0.
        };
        scroll_area.scroll_to(left, top);
    } else if ds.mode == DragMode::DragX {
        let top = scroll_area.scroll_top;
        let left = if scroll_area.visible_size.x > 0. {
            ds.offset + distance.x * scroll_area.content_size.x / scroll_area.visible_size.x
        } else {
            0.
        };
        scroll_area.scroll_to(left, top);
    };
}

fn handle_track_click(scroll_area: &mut ScrollArea, vertical: bool, position: Vec2, rect: Rect) {
    if vertical {
        let page_size = scroll_area.visible_size.y;
        if position.y >= rect.max.y {
            scroll_area.scroll_by(0., page_size);
        } else if position.y < rect.min.y {
            scroll_area.scroll_by(0., -page_size);
        }
    } else {
        let page_size = scroll_area.visible_size.x;
        if position.x >= rect.max.x {
            scroll_area.scroll_by(page_size, 0.);
        } else if position.x < rect.min.x {
            scroll_area.scroll_by(-page_size, 0.);
        }
    }
}
