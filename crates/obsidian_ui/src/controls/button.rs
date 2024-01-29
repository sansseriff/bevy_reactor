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
use static_init::dynamic;

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
#[derive(Clone, PartialEq, Default)]
pub struct ButtonProps<V: ViewTuple + Clone> {
    /// Color variant - default, primary or danger.
    pub variant: ButtonVariant,

    /// Button size.
    pub size: Size,

    /// Whether the button is disabled.
    pub disabled: bool,

    /// The content to display inside the button.
    pub children: V,
}

#[dynamic]
static STYLE_BUTTON: StyleBuilder = StyleBuilder::new(|ss| {
    ss.border(1)
        .display(ui::Display::Flex)
        .justify_content(JustifyContent::Center)
        .align_items(AlignItems::Center)
        .padding_left(12)
        .padding_right(12)
        .border(1)
        .border_color(Color::WHITE);
    // .selector(".size-xxxs", |ss| ss.min_height(Size::Xxxs.height()))
    // .selector(".size-xxs", |ss| ss.min_height(Size::Xxs.height()))
    // .selector(".size-xs", |ss| ss.min_height(Size::Xs.height()))
    // .selector(".size-sm", |ss| ss.min_height(Size::Sm.height()))
    // .selector(".size-md", |ss| ss.min_height(Size::Md.height()))
    // .selector(".size-lg", |ss| ss.min_height(Size::Lg.height()))
    // .selector(".size-xl", |ss| ss.min_height(Size::Xl.height()))
});

#[dynamic]
static STYLE_BUTTON_XXXS: StyleBuilder = StyleBuilder::new(|ss| {
    ss.min_height(Size::Xxxs.height());
});

#[dynamic]
static STYLE_BUTTON_XXS: StyleBuilder = StyleBuilder::new(|ss| {
    ss.min_height(Size::Xxs.height());
});

#[dynamic]
static STYLE_BUTTON_XS: StyleBuilder = StyleBuilder::new(|ss| {
    ss.min_height(Size::Xs.height());
});

#[dynamic]
static STYLE_BUTTON_SM: StyleBuilder = StyleBuilder::new(|ss| {
    ss.min_height(Size::Sm.height());
});

#[dynamic]
static STYLE_BUTTON_MD: StyleBuilder = StyleBuilder::new(|ss| {
    ss.min_height(Size::Md.height());
});

#[dynamic]
static STYLE_BUTTON_LG: StyleBuilder = StyleBuilder::new(|ss| {
    ss.min_height(Size::Lg.height());
});

#[dynamic]
static STYLE_BUTTON_XL: StyleBuilder = StyleBuilder::new(|ss| {
    ss.min_height(Size::Xl.height());
});

/// Construct a button widget.
pub fn button<V: ViewTuple + Clone>(cx: &mut Cx<ButtonProps<V>>) -> Element<NodeBundle> {
    let pressed = cx.create_mutable::<bool>(false);
    let _hover = cx.create_mutable::<bool>(false);

    // Needs to be a local variable so that it can be captured in the event handler.
    let disabled = cx.props.disabled;
    Element::<NodeBundle>::new()
        .named("button")
        .with_styles((
            STYLE_BUTTON.clone(),
            match cx.props.size {
                Size::Xxxs => STYLE_BUTTON_XXXS.clone(),
                Size::Xxs => STYLE_BUTTON_XXS.clone(),
                Size::Xs => STYLE_BUTTON_XS.clone(),
                Size::Sm => STYLE_BUTTON_SM.clone(),
                Size::Md => STYLE_BUTTON_MD.clone(),
                Size::Lg => STYLE_BUTTON_LG.clone(),
                Size::Xl => STYLE_BUTTON_XL.clone(),
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
            On::<Pointer<Click>>::run(move |world: &mut World| {
                // pressed.set(world, true);
                // if !disabled {
                //     writer.send(Clicked {
                //         target: ev.target,
                //         id,
                //     });
                // }
            }),
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
