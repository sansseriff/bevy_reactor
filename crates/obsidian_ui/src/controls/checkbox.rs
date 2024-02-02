use bevy::{
    a11y::{
        accesskit::{NodeBuilder, Role},
        AccessibilityNode,
    },
    prelude::*,
    ui,
};
use bevy_mod_picking::{events::PointerCancel, prelude::*};
use bevy_reactor::*;
// use bevy_tabindex::TabIndex;

use crate::colors;

/// Checkbox properties
#[derive(Default)]
pub struct CheckboxProps<V: ViewTuple + Clone> {
    /// Whether the checkbox is checked.
    pub checked: Signal<bool>,

    /// Whether the checkbox is disabled.
    pub disabled: Signal<bool>,

    /// The content to display inside the button.
    pub label: V,

    /// Additional styles to be applied to the button.
    pub styles: StyleHandle,

    /// Callback called when clicked
    pub on_change: Option<Callback<bool>>,
}

fn style_checkbox(ss: &mut StyleBuilder) {
    ss.border(1)
        .display(ui::Display::Flex)
        .flex_direction(ui::FlexDirection::Row)
        .justify_content(ui::JustifyContent::Center)
        .align_items(ui::AlignItems::Center)
        .align_content(ui::AlignContent::Center)
        .gap(4)
        .color(colors::FOREGROUND);
}

fn style_checkbox_border(ss: &mut StyleBuilder) {
    ss.display(ui::Display::Flex)
        .border(2)
        .border_color(colors::GRAY_700)
        .width(16)
        .height(16);
}

fn style_checkbox_inner(ss: &mut StyleBuilder) {
    ss.display(ui::Display::Flex)
        .position(ui::PositionType::Absolute)
        .left(1)
        .top(1)
        .width(10)
        .height(10);
}

fn style_checkbox_label(ss: &mut StyleBuilder) {
    ss.display(ui::Display::Flex)
        .flex_direction(ui::FlexDirection::Row)
        .justify_content(ui::JustifyContent::FlexStart)
        .align_items(ui::AlignItems::Center)
        .color(colors::FOREGROUND);
}

/// Construct a button widget.
pub fn checkbox<V: ViewTuple + Clone>(cx: &mut Cx<CheckboxProps<V>>) -> Element<NodeBundle> {
    let id = cx.create_entity();
    let pressed = cx.create_mutable::<bool>(false);
    let hovering = cx.create_hover_signal(id);

    let disabled = cx.props.disabled;
    let checked = cx.props.checked;

    Element::<NodeBundle>::for_entity(id)
        .named("checkbox")
        .with_styles((style_checkbox, cx.props.styles.clone()))
        .insert((
            // TabIndex(0),
            AccessibilityNode::from(NodeBuilder::new(Role::CheckBox)),
            {
                let on_click = cx.props.on_change;
                On::<Pointer<Click>>::run(move |world: &mut World| {
                    if !disabled.get(world) {
                        let next_checked = checked.get(world);
                        if let Some(on_click) = on_click {
                            world.run_callback(on_click, !next_checked);
                        }
                    }
                })
            },
            On::<Pointer<DragStart>>::run(move |world: &mut World| {
                if !disabled.get(world) {
                    pressed.set(world, true);
                }
            }),
            On::<Pointer<DragEnd>>::run(move |world: &mut World| {
                if !disabled.get(world) {
                    pressed.set(world, false);
                }
            }),
            On::<Pointer<DragEnter>>::run(move |world: &mut World| {
                if !disabled.get(world) {
                    pressed.set(world, true);
                }
            }),
            On::<Pointer<DragLeave>>::run(move |world: &mut World| {
                if !disabled.get(world) {
                    pressed.set(world, false);
                }
            }),
            On::<Pointer<PointerCancel>>::run(move |world: &mut World| {
                println!("PointerCancel");
                if !disabled.get(world) {
                    pressed.set(world, false);
                }
            }),
        ))
        .children((
            Element::<NodeBundle>::new()
                .with_styles(style_checkbox_border)
                .create_effect(move |cx, ent| {
                    let is_checked = checked.get(cx);
                    let is_pressed = pressed.get(cx);
                    let is_hovering = hovering.get(cx);
                    let color = match (is_checked, is_pressed, is_hovering) {
                        (true, _, _) => Color::RED,
                        (false, true, _) => colors::GRAY_350,
                        (false, false, true) => colors::GRAY_300,
                        (false, false, false) => colors::GRAY_250,
                    };
                    let mut bg = cx.world_mut().get_mut::<BorderColor>(ent).unwrap();
                    bg.0 = color;
                })
                .children(
                    Element::<NodeBundle>::new()
                        .with_styles(style_checkbox_inner)
                        .create_effect(move |cx, ent| {
                            let is_checked = checked.get(cx);
                            let is_pressed = pressed.get(cx);
                            let is_hovering = hovering.get(cx);
                            let color = match (is_checked, is_pressed, is_hovering) {
                                (true, _, _) => colors::GRAY_700,
                                (false, true, _) => colors::GRAY_350,
                                (false, false, true) => colors::GRAY_300,
                                (false, false, false) => colors::GRAY_250,
                            };
                            let mut bg = cx.world_mut().get_mut::<BackgroundColor>(ent).unwrap();
                            bg.0 = color;
                        }),
                ),
            Element::<NodeBundle>::new()
                .with_styles(style_checkbox_label)
                .children(cx.props.label.clone()),
        ))
}
