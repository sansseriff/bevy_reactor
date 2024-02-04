use bevy::prelude::*;
use bevy_mod_picking::prelude::*;
use bevy_reactor::*;

use crate::colors;

/// Properties for slider widget.
#[derive(Default)]
pub struct SliderProps {
    /// Current slider value.
    pub value: Signal<f32>,

    /// Minimum slider value.
    pub min: Signal<f32>,

    /// Maximum slider value.
    pub max: Signal<f32>,

    /// Number of decimal places to round to (0 = integer).
    pub precision: f32,

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

#[derive(Clone, PartialEq, Default, Copy)]
struct DragState {
    dragging: bool,
    offset: f32,
}

fn style_slider(ss: &mut StyleBuilder) {
    ss.border(1)
        .border_color(colors::FOREGROUND)
        .min_width(64)
        .height(24);
}

fn style_slider_value(ss: &mut StyleBuilder) {
    // ss.border(1)
    //     .border_color(colors::FOREGROUND)
    //     .min_width(64)
    //     .height(Size::Md.height());
}

/// Horizontal slider widget
pub fn slider(cx: &mut Cx<SliderProps>) -> Element<NodeBundle> {
    let id = cx.create_entity();
    let hovering = cx.create_hover_signal(id);
    let drag_state = cx.create_mutable::<DragState>(DragState::default());

    // Pain point: Need to capture all props for closures.
    let min = cx.props.min;
    let max = cx.props.max;
    let value = cx.props.value;
    let on_change = cx.props.on_change;
    // let range = max - min;
    // let pos = if range > 0. {
    //     (cx.props.value - cx.props.min) / range
    // } else {
    //     0.
    // }
    // .clamp(0., 1.);

    Element::<NodeBundle>::for_entity(id)
        .with_styles((style_slider, cx.props.style.clone()))
        .insert((
            On::<Pointer<DragStart>>::run(move |world: &mut World| {
                // Save initial value to use as drag offset.
                drag_state.set(
                    world,
                    DragState {
                        dragging: true,
                        offset: value.get(world),
                    },
                );
            }),
            On::<Pointer<DragEnd>>::run(move |world: &mut World| {
                drag_state.set(
                    world,
                    DragState {
                        dragging: false,
                        offset: value.get(world),
                    },
                );
            }),
            On::<Pointer<Drag>>::run(move |world: &mut World| {
                let ds = drag_state.get(world);
                if ds.dragging {
                    let event = world
                        .get_resource::<ListenerInput<Pointer<Drag>>>()
                        .unwrap();
                    let ent = world.entity(id);
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
                        if let Some(on_change) = on_change {
                            world.run_callback(on_change, new_value.clamp(min, max));
                        }
                    }
                }
            }),
        ))
        .children((Element::<NodeBundle>::new().with_styles(style_slider_value),))

    // (SliderChildProps {
    //     percent: pos * 100.,
    //     min,
    //     max,
    //     value,
    //     is_dragging: cx.read_atom(drag_state).dragging,
    // }))
}
