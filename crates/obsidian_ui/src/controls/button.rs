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

use crate::size::Size;

/// The variant determines the button's color scheme
#[derive(Clone, PartialEq, Default, Debug)]
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
    pub disabled: bool,

    /// The content to display inside the button.
    pub children: V,

    /// Callback called when clicked
    pub on_click: Option<CallbackFn>,
}

fn style_button(ss: &mut StyleBuilder) {
    ss.border(1)
        .display(ui::Display::Flex)
        .justify_content(JustifyContent::Center)
        .align_items(AlignItems::Center)
        .padding_left(12)
        .padding_right(12)
        .border(1)
        .border_color(Color::WHITE);
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
    let pressed = cx.create_mutable::<bool>(false);
    let _hover = cx.create_mutable::<bool>(false);

    // Needs to be a local variable so that it can be captured in the event handler.
    let disabled = cx.props.disabled;
    Element::<NodeBundle>::new()
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
        ))
        .create_effect(move |cx, ent| {
            let is_pressed = pressed.get(cx);
            let mut border = cx.world_mut().get_mut::<BorderColor>(ent).unwrap();
            border.0 = if is_pressed {
                Color::LIME_GREEN
            } else {
                Color::RED
            };
        })
        .insert((
            // TabIndex(0),
            AccessibilityNode::from(NodeBuilder::new(Role::Button)),
            {
                let on_click = cx.props.on_click;
                On::<Pointer<Click>>::run(move |world: &mut World| {
                    if !disabled {
                        if let Some(on_click) = on_click {
                            world.run_callback(on_click, ());
                        }
                    }
                })
            },
            On::<Pointer<DragStart>>::run(move |world: &mut World| {
                if !disabled {
                    pressed.set(world, true);
                }
            }),
            On::<Pointer<DragEnd>>::run(move |world: &mut World| {
                if !disabled {
                    pressed.set(world, false);
                }
            }),
            On::<Pointer<DragEnter>>::run(move |world: &mut World| {
                if !disabled {
                    pressed.set(world, true);
                }
            }),
            On::<Pointer<DragLeave>>::run(move |world: &mut World| {
                if !disabled {
                    pressed.set(world, false);
                }
            }),
            On::<Pointer<PointerCancel>>::run(move |world: &mut World| {
                println!("PointerCancel");
                if !disabled {
                    pressed.set(world, false);
                }
            }),
        ))
        .children(cx.props.children.clone())
}
