use bevy::{prelude::*, ui};
use bevy_mod_picking::prelude::*;
use bevy_reactor::*;
use bevy_reactor_signals::{Callback, Cx, IntoSignal, RunContextSetup, RunContextWrite, Signal};

use crate::{colors, cursor::StyleBuilderCursor, hooks::UseElementRect, RoundedCorners};

use super::IconButton;

#[derive(Clone, PartialEq, Default, Copy)]
enum DragType {
    #[default]
    None = 0,
    Dragging,
}

#[derive(Clone, PartialEq, Default, Copy)]
struct DragState {
    dragging: DragType,
    offset: f32,
    was_dragged: bool,
}

fn style_spinbox(ss: &mut StyleBuilder) {
    ss.min_width(24)
        .height(20)
        .background_color(colors::U1)
        .border_radius(5);
}

fn style_overlay(ss: &mut StyleBuilder) {
    ss.display(ui::Display::Flex)
        .flex_direction(ui::FlexDirection::Row)
        .align_items(ui::AlignItems::Center)
        .position(ui::PositionType::Absolute)
        .left(0)
        .top(0)
        .bottom(0)
        .right(0)
        .cursor(CursorIcon::ColResize);
}

fn style_spinbox_label(ss: &mut StyleBuilder) {
    ss.flex_grow(1.)
        .display(ui::Display::Flex)
        .flex_direction(ui::FlexDirection::Row)
        .align_items(ui::AlignItems::Center)
        .justify_content(ui::JustifyContent::FlexEnd)
        .height(ui::Val::Percent(100.))
        .font("obsidian_ui://fonts/Open_Sans/static/OpenSans-Medium.ttf")
        .font_size(16)
        .overflow(ui::OverflowAxis::Hidden)
        .padding((3, 0))
        .color(colors::FOREGROUND);
}

fn style_spinbox_button(ss: &mut StyleBuilder) {
    ss.height(20.).padding(0).max_width(12).flex_grow(0.2);
}

/// A numeric spinbox. This is a widget that allows the user to input a number by typing, using
/// arrow buttons, or dragging. It is preferred over a slider in two cases:
/// * The range of values is large or unbounded, making it difficult to select a specific value
///   with a slider.
/// * There is limited horizontal space available.
pub struct SpinBox {
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

impl SpinBox {
    /// Create a new spinbox.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the current spinbox value.
    pub fn value(mut self, value: impl IntoSignal<f32>) -> Self {
        self.value = value.into_signal();
        self
    }

    /// Set the minimum spinbox value.
    pub fn min(mut self, min: impl IntoSignal<f32>) -> Self {
        self.min = min.into_signal();
        self
    }

    /// Set the maximum spinbox value.
    pub fn max(mut self, max: impl IntoSignal<f32>) -> Self {
        self.max = max.into_signal();
        self
    }

    /// Set the number of decimal places to round to (0 = integer).
    pub fn precision(mut self, precision: usize) -> Self {
        self.precision = precision;
        self
    }

    /// Set the amount to increment when using arrow buttons.
    pub fn step(mut self, step: f32) -> Self {
        self.step = step;
        self
    }

    /// Set whether the spinbox is disabled.
    pub fn disabled(mut self, disabled: impl IntoSignal<bool>) -> Self {
        self.disabled = disabled.into_signal();
        self
    }

    /// Set the signal which returns the value formatted as a string. If `None`, then a default
    /// formatter will be used.
    pub fn formatted_value(mut self, formatted_value: impl IntoSignal<String>) -> Self {
        self.formatted_value = Some(formatted_value.into_signal());
        self
    }

    /// Set the style handle for the spinbox root element.
    pub fn style<S: StyleTuple + 'static>(mut self, style: S) -> Self {
        self.style = style.into_handle();
        self
    }

    /// Set the callback called when value changes.
    pub fn on_change(mut self, on_change: Callback<f32>) -> Self {
        self.on_change = Some(on_change);
        self
    }
}

impl Default for SpinBox {
    fn default() -> Self {
        Self {
            value: Signal::Constant(0.),
            min: Signal::Constant(f32::MIN),
            max: Signal::Constant(f32::MAX),
            precision: 0,
            step: 1.,
            disabled: Signal::Constant(false),
            formatted_value: None,
            style: StyleHandle::default(),
            on_change: None,
        }
    }
}

impl ViewTemplate for SpinBox {
    fn create(&self, cx: &mut Cx) -> impl IntoView {
        let spinbox_id = cx.create_entity();
        let drag_state = cx.create_mutable::<DragState>(DragState::default());
        let rect = cx.use_element_rect(spinbox_id);
        let show_buttons = cx.create_derived(move |cx| {
            let rect = rect.get(cx);
            rect.width() >= 48.
        });

        // Pain point: Need to capture all props for closures.
        let min = self.min;
        let max = self.max;
        let value = self.value;
        let precision = self.precision;
        let step = self.step;
        let on_change = self.on_change;

        let dec_disabled = cx.create_derived(move |cx| value.get(cx) <= min.get(cx));
        let dec_click = cx.create_callback(move |cx, _| {
            let min = min.get(cx);
            let max = max.get(cx);
            let value = value.get(cx) - step;
            if let Some(on_change) = on_change {
                cx.run_callback(on_change, value.clamp(min, max));
            }
        });
        let inc_disabled = cx.create_derived(move |cx| value.get(cx) >= max.get(cx));
        let inc_click = cx.create_callback(move |cx, _| {
            let min = min.get(cx);
            let max = max.get(cx);
            let value = value.get(cx) + step;
            if let Some(on_change) = on_change {
                cx.run_callback(on_change, value.clamp(min, max));
            }
        });

        Element::<NodeBundle>::for_entity(spinbox_id)
            .style((style_spinbox, self.style.clone()))
            .children((Element::<NodeBundle>::new()
                .named("SpinBox")
                .style(style_overlay)
                .children((
                    Cond::new(
                        show_buttons,
                        move || {
                            IconButton::new("obsidian_ui://icons/chevron_left.png")
                                .corners(RoundedCorners::Left)
                                .style(style_spinbox_button)
                                .minimal(true)
                                .disabled(dec_disabled)
                                .on_click(dec_click)
                        },
                        || (),
                    ),
                    Element::<NodeBundle>::new()
                        .style(style_spinbox_label)
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
                                        was_dragged: false,
                                    },
                                );
                            }),
                            On::<Pointer<DragEnd>>::run(move |world: &mut World| {
                                let ds = drag_state.get(world);
                                if ds.dragging == DragType::Dragging {
                                    if !ds.was_dragged {
                                        println!("was not dragged");
                                    }
                                    drag_state.set(
                                        world,
                                        DragState {
                                            dragging: DragType::None,
                                            offset: value.get(world),
                                            was_dragged: false,
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
                                    let min = min.get(world);
                                    let max = max.get(world);
                                    let new_value = ds.offset
                                        + ((event.distance.x - event.distance.y) * 0.1 * step);
                                    let rounding = f32::powi(10., precision as i32);
                                    let value = value.get(world);
                                    let new_value = (new_value * rounding).round() / rounding;
                                    if value != new_value {
                                        if !ds.was_dragged {
                                            drag_state.set(
                                                world,
                                                DragState {
                                                    was_dragged: true,
                                                    ..ds
                                                },
                                            );
                                        }
                                        if let Some(on_change) = on_change {
                                            world
                                                .run_callback(on_change, new_value.clamp(min, max));
                                        }
                                    }
                                }
                            }),
                        ))
                        .children((text_computed({
                            move |cx| {
                                let value = value.get(cx);
                                format!("{:.*}", precision, value)
                            }
                        }),)),
                    Cond::new(
                        show_buttons,
                        move || {
                            IconButton::new("obsidian_ui://icons/chevron_right.png")
                                .corners(RoundedCorners::Right)
                                .minimal(true)
                                .style(style_spinbox_button)
                                .disabled(inc_disabled)
                                .on_click(inc_click)
                        },
                        || (),
                    ),
                )),))
    }
}
