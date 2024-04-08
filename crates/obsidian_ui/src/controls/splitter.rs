use bevy::{color::Luminance, prelude::*, ui};
use bevy_mod_picking::{events::PointerCancel, prelude::*};
use bevy_reactor::*;

use crate::colors;

/// The direction of the splitter. Represents the direction of the bar, not the items being split.
#[derive(Clone, PartialEq, Default)]
pub enum SplitterDirection {
    /// The splitter bar runs horizontally, and splits the items above and below it.
    Horizontal,

    /// The splitter bar runs vertically, and splits the items to the left and right of it.
    #[default]
    Vertical,
}

#[derive(Clone, PartialEq, Default, Copy)]
struct DragState {
    dragging: bool,
    offset: f32,
}

fn style_vsplitter(ss: &mut StyleBuilder) {
    ss.align_items(ui::AlignItems::Center)
        .justify_content(ui::JustifyContent::Center)
        .display(ui::Display::Flex)
        .flex_direction(ui::FlexDirection::Column)
        .gap(8)
        .width(9);
}

// The decorative handle inside the splitter.
fn style_vsplitter_inner(ss: &mut StyleBuilder) {
    ss.display(ui::Display::Flex)
        .width(3)
        // .pointer_events(PointerEvents::None)
        .height(ui::Val::Percent(20.));
}

/// Splitter bar which can be dragged
pub struct Splitter {
    /// The current split value.
    pub value: Signal<f32>,

    /// Whether the splitter bar runs horizontally or vertically.
    pub direction: SplitterDirection,

    /// Callback involved with the new split value.
    pub on_change: Callback<f32>,
}

impl ViewTemplate for Splitter {
    fn create(&self, cx: &mut Cx) -> impl View + Send + Sync + 'static {
        let id = cx.create_entity();
        let hovering = cx.create_hover_signal(id);
        let drag_state = cx.create_mutable::<DragState>(DragState::default());
        let current_offset = self.value;
        Element::<NodeBundle>::for_entity(id)
            .named("v_splitter")
            // .class_names(CLS_DRAG.if_true(cx.read_atom(drag_state).dragging))
            .with_styles(style_vsplitter)
            .insert((
                On::<Pointer<DragStart>>::run(move |world: &mut World| {
                    // Save initial value to use as drag offset.
                    drag_state.set(
                        world,
                        DragState {
                            dragging: true,
                            offset: current_offset.get(world),
                        },
                    );
                }),
                On::<Pointer<DragEnd>>::run(move |world: &mut World| {
                    drag_state.set(
                        world,
                        DragState {
                            dragging: false,
                            offset: current_offset.get(world),
                        },
                    );
                }),
                On::<Pointer<Drag>>::run({
                    let on_change = self.on_change;
                    move |world: &mut World| {
                        let event = world
                            .get_resource::<ListenerInput<Pointer<Drag>>>()
                            .unwrap();
                        let ev = event.distance;
                        let ds = drag_state.get(world);
                        if ds.dragging {
                            world.run_callback(on_change, ev.x + ds.offset);
                        }
                    }
                }),
                On::<Pointer<PointerCancel>>::run(move |world: &mut World| {
                    println!("Splitter Cancel");
                    drag_state.set(
                        world,
                        DragState {
                            dragging: false,
                            offset: current_offset.get(world),
                        },
                    );
                }),
            ))
            .with_children(
                Element::<NodeBundle>::new()
                    .with_styles(style_vsplitter_inner)
                    .create_effect(move |cx, ent| {
                        // Color change on hover / drag
                        let ds = drag_state.get(cx);
                        let is_hovering = hovering.get(cx);
                        let color = match (ds.dragging, is_hovering) {
                            (true, _) => colors::U3.lighter(0.05),
                            (false, true) => colors::U3.lighter(0.02),
                            (false, false) => colors::U3,
                        };
                        let mut bg = cx.world_mut().get_mut::<BackgroundColor>(ent).unwrap();
                        bg.0 = color.into();
                    }),
            )
    }
}
