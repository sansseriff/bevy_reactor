use bevy::{
    a11y::{
        accesskit::{NodeBuilder, Role},
        AccessibilityNode,
    },
    prelude::*,
    ui,
};
use bevy_color::{LinearRgba, Luminance};
use bevy_mod_picking::{events::PointerCancel, prelude::*};
use bevy_reactor::*;
// use bevy_tabindex::TabIndex;

use crate::{
    colors,
    focus::{KeyPressEvent, TabIndex},
    hooks::CreateFocusSignal,
    materials::RoundedRectMaterial,
    RoundedCorners,
};

/// Checkbox properties
#[derive(Default)]
pub struct CheckboxProps {
    /// Whether the checkbox is checked.
    pub checked: Signal<bool>,

    /// Whether the checkbox is disabled.
    pub disabled: Signal<bool>,

    /// The content to display inside the button.
    pub label: ViewHandle,

    /// Additional styles to be applied to the button.
    pub styles: StyleHandle,

    /// Callback called when clicked
    pub on_change: Option<Callback<bool>>,

    /// The tab index of the button (default 0).
    pub tab_index: i32,
}

fn style_checkbox(ss: &mut StyleBuilder) {
    ss.display(ui::Display::Flex)
        .flex_direction(ui::FlexDirection::Row)
        .justify_content(ui::JustifyContent::Center)
        .align_items(ui::AlignItems::Center)
        .align_content(ui::AlignContent::Center)
        .gap(4)
        .color(colors::FOREGROUND);
}

fn style_checkbox_border(ss: &mut StyleBuilder) {
    ss.display(ui::Display::Flex).width(16).height(16);
}

fn style_checkbox_inner(ss: &mut StyleBuilder) {
    ss.display(ui::Display::Flex)
        .background_image("obsidian_ui://textures/checkmark.png")
        .background_color(colors::FOREGROUND)
        .position(ui::PositionType::Absolute)
        .left(2)
        .top(2)
        .width(12)
        .height(12);
}

fn style_checkbox_label(ss: &mut StyleBuilder) {
    ss.display(ui::Display::Flex)
        .flex_direction(ui::FlexDirection::Row)
        .justify_content(ui::JustifyContent::FlexStart)
        .align_items(ui::AlignItems::Center)
        .color(colors::FOREGROUND);
}

/// Construct a button widget.
pub fn checkbox(cx: &mut Cx<CheckboxProps>) -> Element<NodeBundle> {
    let id = cx.create_entity();
    let pressed = cx.create_mutable::<bool>(false);
    let hovering = cx.create_hover_signal(id);
    let focused = cx.create_focus_signal(id);

    let disabled = cx.props.disabled;
    let checked = cx.props.checked;

    let mut ui_materials = cx
        .world_mut()
        .get_resource_mut::<Assets<RoundedRectMaterial>>()
        .unwrap();
    let material = ui_materials.add(RoundedRectMaterial {
        color: colors::U3.into(),
        radius: RoundedCorners::All.to_vec(8.0),
    });

    Element::<NodeBundle>::for_entity(id)
        .named("checkbox")
        .with_styles((style_checkbox, cx.props.styles.clone()))
        .insert((
            TabIndex(cx.props.tab_index),
            AccessibilityNode::from(NodeBuilder::new(Role::CheckBox)),
            {
                let on_change = cx.props.on_change;
                On::<Pointer<Click>>::run(move |world: &mut World| {
                    if !disabled.get(world) {
                        let next_checked = checked.get(world);
                        if let Some(on_click) = on_change {
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
            On::<KeyPressEvent>::run({
                let on_change = cx.props.on_change;
                move |world: &mut World| {
                    if !disabled.get(world) {
                        let mut event = world
                            .get_resource_mut::<ListenerInput<KeyPressEvent>>()
                            .unwrap();
                        if event.key_code == KeyCode::Return || event.key_code == KeyCode::Space {
                            event.stop_propagation();
                            let next_checked = checked.get(world);
                            if let Some(on_click) = on_change {
                                world.run_callback(on_click, !next_checked);
                            }
                        }
                    }
                }
            }),
        ))
        .children((
            Element::<MaterialNodeBundle<RoundedRectMaterial>>::new()
                .with_styles(style_checkbox_border)
                .insert(material.clone())
                .create_effect(move |cx, _| {
                    let is_checked = checked.get(cx);
                    let is_pressed = pressed.get(cx);
                    let is_hovering = hovering.get(cx);
                    let color = match (is_checked, is_pressed, is_hovering) {
                        (true, true, _) => colors::ACCENT.darker(0.1),
                        (true, false, true) => colors::ACCENT.darker(0.15),
                        (true, _, _) => colors::ACCENT.darker(0.2),
                        (false, true, _) => colors::U1.lighter(0.005),
                        (false, false, true) => colors::U1.lighter(0.002),
                        (false, false, false) => colors::U1,
                    };
                    let mut ui_materials = cx
                        .world_mut()
                        .get_resource_mut::<Assets<RoundedRectMaterial>>()
                        .unwrap();
                    let material = ui_materials.get_mut(material.clone()).unwrap();
                    material.color = LinearRgba::from(color).into();
                    // let mut bg = cx.world_mut().get_mut::<BackgroundColor>(ent).unwrap();
                    // bg.0 = color.into();
                })
                .create_effect(move |cx, entt| {
                    let is_focused = focused.get(cx);
                    let mut entt = cx.world_mut().entity_mut(entt);
                    match is_focused {
                        true => {
                            entt.insert(Outline {
                                color: colors::FOCUS.into(),
                                offset: ui::Val::Px(2.0),
                                width: ui::Val::Px(2.0),
                            });
                        }
                        false => {
                            entt.remove::<Outline>();
                        }
                    };
                })
                .children(cond(
                    move |cx| checked.get(cx),
                    move || Element::<NodeBundle>::new().with_styles(style_checkbox_inner),
                    || (),
                )),
            Element::<NodeBundle>::new()
                .with_styles(style_checkbox_label)
                .with_child(&cx.props.label),
        ))
}
