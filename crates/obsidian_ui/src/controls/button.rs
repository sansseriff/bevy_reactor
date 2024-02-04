use bevy::{
    a11y::{
        accesskit::{NodeBuilder, Role},
        AccessibilityNode,
    },
    prelude::*,
    ui,
};
use bevy_color::ColorOps;
use bevy_mod_picking::{events::PointerCancel, prelude::*};
use bevy_reactor::*;
// use bevy_tabindex::TabIndex;

use crate::{colors, rounded_rect::RoundedRectMaterial, size::Size};

/// The variant determines the button's color scheme
#[derive(Clone, Copy, PartialEq, Default, Debug)]
pub enum ButtonVariant {
    /// The default apperance.
    #[default]
    Default,

    /// A more prominent, "call to action", appearance.
    Primary,

    /// An appearance indicating a potentially dangerous action.
    Danger,
}

/// Button properties
#[derive(Default)]
pub struct ButtonProps<V: ViewTuple + Clone> {
    /// Color variant - default, primary or danger.
    pub variant: ButtonVariant,

    /// Button size.
    pub size: Size,

    /// Whether the button is disabled.
    pub disabled: Option<Signal<bool>>,

    /// The content to display inside the button.
    pub children: V,

    /// Additional styles to be applied to the button.
    pub styles: StyleHandle,

    /// Callback called when clicked
    pub on_click: Option<Callback>,
}

fn style_button(ss: &mut StyleBuilder) {
    ss.border(1)
        .display(ui::Display::Flex)
        .flex_direction(ui::FlexDirection::Row)
        .justify_content(ui::JustifyContent::Center)
        .align_items(ui::AlignItems::Center)
        .align_content(ui::AlignContent::Center)
        .padding((12, 0))
        .border(0)
        .color(colors::FOREGROUND);
}

fn style_button_bg(ss: &mut StyleBuilder) {
    ss.display(ui::Display::Grid)
        .position(ui::PositionType::Absolute)
        .left(0)
        .right(0)
        .top(0)
        .bottom(0);
}

fn style_button_xxxs(ss: &mut StyleBuilder) {
    ss.min_height(Size::Xxxs.height());
}

fn style_button_xxs(ss: &mut StyleBuilder) {
    ss.min_height(Size::Xxs.height());
}

fn style_button_xs(ss: &mut StyleBuilder) {
    ss.min_height(Size::Xs.height());
}

fn style_button_sm(ss: &mut StyleBuilder) {
    ss.min_height(Size::Sm.height());
}

fn style_button_md(ss: &mut StyleBuilder) {
    ss.min_height(Size::Md.height());
}

fn style_button_lg(ss: &mut StyleBuilder) {
    ss.min_height(Size::Lg.height());
}

fn style_button_xl(ss: &mut StyleBuilder) {
    ss.min_height(Size::Xl.height());
}

/// Construct a button widget.
pub fn button<V: ViewTuple + Clone>(cx: &mut Cx<ButtonProps<V>>) -> Element<NodeBundle> {
    let id = cx.create_entity();
    let pressed = cx.create_mutable::<bool>(false);
    let hovering = cx.create_hover_signal(id);

    let disabled = cx.props.disabled;
    let disabled = cx.create_derived(move |cc| disabled.map(|s| s.get(cc)).unwrap_or(false));

    let mut ui_materials = cx
        .world_mut()
        .get_resource_mut::<Assets<RoundedRectMaterial>>()
        .unwrap();
    let material = ui_materials.add(RoundedRectMaterial {
        color: colors::U3.into(),
        radius: Vec4::new(4., 4., 4., 4.),
    });

    Element::<NodeBundle>::for_entity(id)
        .named("button")
        .with_styles((
            style_button,
            match cx.props.size {
                Size::Xxxs => style_button_xxxs,
                Size::Xxs => style_button_xxs,
                Size::Xs => style_button_xs,
                Size::Sm => style_button_sm,
                Size::Md => style_button_md,
                Size::Lg => style_button_lg,
                Size::Xl => style_button_xl,
            },
            cx.props.styles.clone(),
        ))
        .insert((
            // TabIndex(0),
            AccessibilityNode::from(NodeBuilder::new(Role::Button)),
            {
                let on_click = cx.props.on_click;
                On::<Pointer<Click>>::run(move |world: &mut World| {
                    if !disabled.get(world) {
                        if let Some(on_click) = on_click {
                            world.run_callback(on_click, ());
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
            Element::<MaterialNodeBundle<RoundedRectMaterial>>::new()
                .insert(material.clone())
                .with_styles(style_button_bg)
                .create_effect(move |cx, _| {
                    let is_pressed = pressed.get(cx);
                    let is_hovering = hovering.get(cx);
                    let color = match (is_pressed, is_hovering) {
                        (true, _) => colors::U3.lighten(0.05),
                        (false, true) => colors::U3.lighten(0.01),
                        (false, false) => colors::U3,
                    };
                    let mut ui_materials = cx
                        .world_mut()
                        .get_resource_mut::<Assets<RoundedRectMaterial>>()
                        .unwrap();
                    let material = ui_materials.get_mut(material.clone()).unwrap();
                    material.color = color.to_linear().into();
                }),
            cx.props.children.clone(),
        ))
}
