use bevy::{prelude::*, ui};
use bevy_color::LuminanceOps;
use bevy_mod_picking::prelude::*;
use bevy_reactor::*;

use crate::colors;

/// Properties for slider widget.
pub struct SliderProps {
    /// Current slider value.
    pub value: Signal<f32>,

    /// Minimum slider value.
    pub min: Signal<f32>,

    /// Maximum slider value.
    pub max: Signal<f32>,

    /// Number of decimal places to round to (0 = integer).
    pub precision: usize,

    /// Amount to increment when using arrow buttons.
    pub step: f32,

    /// Whether the slider is disabled.
    pub disabled: Signal<bool>,

    /// Signal which returns the value formatted as a string. It `None`, then a default
    /// formatter will be used.
    pub formatted_value: Option<Signal<String>>,

    /// Style handle for slider root element.
    pub style: StyleHandle,

    /// Callback called when value changes
    pub on_change: Option<Callback<f32>>,
}

impl Default for SliderProps {
    fn default() -> Self {
        Self {
            value: Signal::Constant(0.),
            min: Signal::Constant(0.),
            max: Signal::Constant(1.),
            precision: 0,
            step: 1.,
            disabled: Signal::Constant(false),
            formatted_value: None,
            style: StyleHandle::default(),
            on_change: None,
        }
    }
}

#[derive(Clone, PartialEq, Default, Copy)]
enum DragType {
    #[default]
    None = 0,
    Dragging,
    HoldDecrement,
    HoldIncrement,
}

#[derive(Clone, PartialEq, Default, Copy)]
struct DragState {
    dragging: DragType,
    offset: f32,
}

fn style_slider(ss: &mut StyleBuilder) {
    ss.border(1)
        .background_color(colors::U1)
        .min_width(64)
        .height(20);
}

fn style_value_bar(ss: &mut StyleBuilder) {
    ss.background_color(colors::U3)
        .position(ui::PositionType::Absolute)
        .left(0)
        .top(0)
        .bottom(0);
}

fn style_overlay(ss: &mut StyleBuilder) {
    ss.display(ui::Display::Flex)
        .flex_direction(ui::FlexDirection::Row)
        .align_items(ui::AlignItems::Center)
        .position(ui::PositionType::Absolute)
        .left(0)
        .top(0)
        .bottom(0)
        .right(0);
}

fn style_button(ss: &mut StyleBuilder) {
    ss.width(16)
        .height(ui::Val::Percent(100.))
        .display(ui::Display::Flex)
        .flex_direction(ui::FlexDirection::Row)
        .align_items(ui::AlignItems::Center)
        .justify_content(ui::JustifyContent::Center)
        .font("obsidian_ui://fonts/Open_Sans/static/OpenSans-Regular.ttf")
        .font_size(16);
}

fn style_button_icon(ss: &mut StyleBuilder) {
    ss.height(16)
        .width(16)
        .background_color(colors::FOREGROUND)
        .background_image("obsidian_ui://icons/chevron_left.png")
        .pointer_events(false);
}

fn style_button_icon_left(ss: &mut StyleBuilder) {
    ss.background_image("obsidian_ui://icons/chevron_left.png");
}

fn style_button_icon_right(ss: &mut StyleBuilder) {
    ss.background_image("obsidian_ui://icons/chevron_right.png");
}

fn style_label(ss: &mut StyleBuilder) {
    ss.flex_grow(1.)
        .display(ui::Display::Flex)
        .flex_direction(ui::FlexDirection::Row)
        .align_items(ui::AlignItems::Center)
        .justify_content(ui::JustifyContent::Center)
        .height(ui::Val::Percent(100.))
        .border_color(colors::DESTRUCTIVE)
        .font("obsidian_ui://fonts/Open_Sans/static/OpenSans-Medium.ttf")
        .font_size(16);
}

/// Horizontal slider widget
pub fn slider(cx: &mut Cx<SliderProps>) -> Element<NodeBundle> {
    let slider_id = cx.create_entity();
    let hovering = cx.create_hover_signal(slider_id);
    let drag_state = cx.create_mutable::<DragState>(DragState::default());

    // Pain point: Need to capture all props for closures.
    let min = cx.props.min;
    let max = cx.props.max;
    let value = cx.props.value;
    let precision = cx.props.precision;
    let step = cx.props.step;
    let on_change = cx.props.on_change;

    Element::<NodeBundle>::for_entity(slider_id)
        .with_styles((style_slider, cx.props.style.clone()))
        .insert((
            On::<Pointer<DragStart>>::run(move |world: &mut World| {
                // Save initial value to use as drag offset.
                let mut event = world
                    .get_resource_mut::<ListenerInput<Pointer<DragStart>>>()
                    .unwrap();
                event.stop_propagation();
                drag_state.set(
                    world,
                    DragState {
                        dragging: DragType::Dragging,
                        offset: value.get(world),
                    },
                );
            }),
            On::<Pointer<DragEnd>>::run(move |world: &mut World| {
                let ds = drag_state.get(world);
                if ds.dragging == DragType::Dragging {
                    drag_state.set(
                        world,
                        DragState {
                            dragging: DragType::None,
                            offset: value.get(world),
                        },
                    );
                }
            }),
            On::<Pointer<Drag>>::run(move |world: &mut World| {
                let ds = drag_state.get(world);
                if ds.dragging == DragType::Dragging {
                    let event = world
                        .get_resource::<ListenerInput<Pointer<Drag>>>()
                        .unwrap();
                    let ent = world.entity(slider_id);
                    let node = ent.get::<Node>();
                    let transform = ent.get::<GlobalTransform>();
                    if let (Some(node), Some(transform)) = (node, transform) {
                        // Measure node width and slider value.
                        let slider_width = node.logical_rect(transform).width();
                        let min = min.get(world);
                        let max = max.get(world);
                        let range = max - min;
                        let new_value = if range > 0. {
                            ds.offset + (event.distance.x * range) / slider_width
                        } else {
                            min + range * 0.5
                        };
                        let rounding = f32::powi(10., precision as i32);
                        let new_value = (new_value * rounding).round() / rounding;
                        if let Some(on_change) = on_change {
                            world.run_callback(on_change, new_value.clamp(min, max));
                        }
                    }
                }
            }),
        ))
        .children((
            Element::<NodeBundle>::new()
                .with_styles(style_value_bar)
                .create_effect(move |cx, ent| {
                    let ds = drag_state.get(cx);
                    let is_hovering = hovering.get(cx);
                    let color = match (ds.dragging, is_hovering) {
                        (DragType::Dragging, _) => colors::U3.lighter(0.03),
                        (_, true) => colors::U3.lighter(0.01),
                        (_, false) => colors::U3,
                    };
                    let mut bg = cx.world_mut().get_mut::<BackgroundColor>(ent).unwrap();
                    bg.0 = color.into();
                })
                .create_effect(move |cx, ent| {
                    let min = min.get(cx);
                    let max = max.get(cx);
                    let value = value.get(cx);
                    let percent = if max > min {
                        (value - min) / (max - min)
                    } else {
                        0.
                    };

                    let mut style = cx.world_mut().get_mut::<Style>(ent).unwrap();
                    style.width = ui::Val::Percent(percent * 100.);
                }),
            Element::<NodeBundle>::new()
                .with_styles(style_overlay)
                .children((
                    slider_button.bind(SliderButtonProps {
                        value,
                        min,
                        max,
                        step: -step,
                        hovering,
                        on_change,
                        drag_state,
                    }),
                    Element::<NodeBundle>::new()
                        .with_styles(style_label)
                        .children(text_computed({
                            move |cx| {
                                let value = value.get(cx);
                                format!("{:.*}", precision, value)
                            }
                        })),
                    slider_button.bind(SliderButtonProps {
                        value,
                        min,
                        max,
                        step,
                        hovering,
                        on_change,
                        drag_state,
                    }),
                )),
        ))
}

struct SliderButtonProps {
    value: Signal<f32>,
    min: Signal<f32>,
    max: Signal<f32>,
    step: f32,
    hovering: Signal<bool>,
    on_change: Option<Callback<f32>>,
    drag_state: Mutable<DragState>,
}

fn slider_button(cx: &mut Cx<SliderButtonProps>) -> Element<NodeBundle> {
    let button_id = cx.create_entity();
    let button_hovering = cx.create_hover_signal(button_id);
    let hovering = cx.props.hovering;
    let min = cx.props.min;
    let max = cx.props.max;
    let value = cx.props.value;
    let step = cx.props.step;
    let on_change = cx.props.on_change;
    let drag_state = cx.props.drag_state;
    let drag_type = if step > 0.0 {
        DragType::HoldIncrement
    } else {
        DragType::HoldDecrement
    };

    Element::<NodeBundle>::for_entity(button_id)
        .with_styles(style_button)
        .insert((
            On::<Pointer<DragStart>>::run(move |world: &mut World| {
                let mut event = world
                    .get_resource_mut::<ListenerInput<Pointer<DragStart>>>()
                    .unwrap();
                event.stop_propagation();
                drag_state.set(
                    world,
                    DragState {
                        dragging: drag_type,
                        offset: value.get(world),
                    },
                );
                let min = min.get(world);
                let max = max.get(world);
                let value = value.get(world) + step;
                if let Some(on_change) = on_change {
                    world.run_callback(on_change, value.clamp(min, max));
                }
            }),
            On::<Pointer<DragEnd>>::run(move |world: &mut World| {
                let mut event = world
                    .get_resource_mut::<ListenerInput<Pointer<DragEnd>>>()
                    .unwrap();
                event.stop_propagation();
                drag_state.set(
                    world,
                    DragState {
                        dragging: DragType::None,
                        offset: value.get(world),
                    },
                );
            }),
            On::<Pointer<DragEnter>>::run(move |world: &mut World| {
                let ds = drag_state.get(world);
                let mut event = world
                    .get_resource_mut::<ListenerInput<Pointer<DragEnter>>>()
                    .unwrap();
                if ds.dragging == DragType::None {
                    event.stop_propagation();
                    drag_state.set(
                        world,
                        DragState {
                            dragging: drag_type,
                            offset: value.get(world),
                        },
                    );
                }
            }),
            On::<Pointer<DragLeave>>::run(move |world: &mut World| {
                let ds = drag_state.get(world);
                if ds.dragging == drag_type {
                    let mut event = world
                        .get_resource_mut::<ListenerInput<Pointer<DragLeave>>>()
                        .unwrap();
                    event.stop_propagation();
                    drag_state.set(
                        world,
                        DragState {
                            dragging: DragType::None,
                            offset: value.get(world),
                        },
                    );
                }
            }),
        ))
        .children(
            Element::<NodeBundle>::new()
                .with_styles((
                    style_button_icon,
                    if step > 0.0 {
                        style_button_icon_right
                    } else {
                        style_button_icon_left
                    },
                ))
                .create_effect(move |cx, ent| {
                    let ds = drag_state.get(cx);
                    let is_hovering = hovering.get(cx) && step != 0.0;
                    let is_hovering_inc = button_hovering.get(cx);
                    let color = match (ds.dragging, is_hovering, is_hovering_inc) {
                        (DragType::HoldIncrement, _, _) if step > 0.0 => colors::FOREGROUND,
                        (DragType::HoldDecrement, _, _) if step < 0.0 => colors::FOREGROUND,
                        (DragType::Dragging, _, _) => colors::TRANSPARENT,
                        (_, true, true) => colors::U4.lighter(0.1),
                        (_, true, false) => colors::U4,
                        _ => colors::TRANSPARENT,
                    };
                    let mut bg = cx.world_mut().get_mut::<BackgroundColor>(ent).unwrap();
                    bg.0 = color.into();
                }),
        )
}
