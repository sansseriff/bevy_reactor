use crate::{
    focus::{AutoFocus, KeyPressEvent, TabIndex},
    hooks::CreateFocusSignal,
    RoundedCorners,
};
use bevy::{
    a11y::{
        accesskit::{NodeBuilder, Role},
        AccessibilityNode, Focus,
    },
    color::Luminance,
    prelude::*,
    ui,
};
use bevy_mod_picking::{events::PointerCancel, prelude::*};
use bevy_reactor::*;

use crate::{colors, size::Size};

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

    /// A button that is in a "toggled" state.
    Selected,
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

/// Button widget
#[derive(Default)]
pub struct Button {
    /// Color variant - default, primary or danger.
    pub variant: Signal<ButtonVariant>,

    /// Button size.
    pub size: Size,

    /// Whether the button is disabled.
    pub disabled: Signal<bool>,

    /// The content to display inside the button.
    pub children: ViewHandle,

    /// Additional styles to be applied to the button.
    pub style: StyleHandle,

    /// Callback called when clicked
    pub on_click: Option<Callback>,

    /// The tab index of the button (default 0).
    pub tab_index: i32,

    /// Which corners to render rounded.
    pub corners: RoundedCorners,

    /// If true, set focus to this button when it's added to the UI.
    pub autofocus: bool,
}

impl ViewTemplate for Button {
    fn create(&self, cx: &mut Cx) -> impl View + Send + Sync + 'static {
        let id = cx.create_entity();
        let variant = self.variant;
        let pressed = cx.create_mutable::<bool>(false);
        let hovering = cx.create_hover_signal(id);
        let focused = cx.create_focus_visible_signal(id);

        let disabled = self.disabled;
        let corners = self.corners;

        let size = self.size;

        Element::<NodeBundle>::for_entity(id)
            .named("button")
            .with_styles((
                style_button,
                move |ss: &mut StyleBuilder| {
                    ss.min_height(size.height());
                },
                self.style.clone(),
            ))
            .insert((
                TabIndex(self.tab_index),
                AccessibilityNode::from(NodeBuilder::new(Role::Button)),
                {
                    let on_click = self.on_click;
                    On::<Pointer<Click>>::run(move |world: &mut World| {
                        let mut focus = world.get_resource_mut::<Focus>().unwrap();
                        focus.0 = Some(id);
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
                On::<KeyPressEvent>::run({
                    let on_click = self.on_click;
                    move |world: &mut World| {
                        if !disabled.get(world) {
                            let mut event = world
                                .get_resource_mut::<ListenerInput<KeyPressEvent>>()
                                .unwrap();
                            if !event.repeat
                                && (event.key_code == KeyCode::Enter
                                    || event.key_code == KeyCode::Space)
                            {
                                event.stop_propagation();
                                if let Some(on_click) = on_click {
                                    world.run_callback(on_click, ());
                                }
                            }
                        }
                    }
                }),
            ))
            .insert_if(self.autofocus, AutoFocus)
            .with_children((
                Element::<NodeBundle>::new()
                    .with_styles(style_button_bg)
                    .insert(corners.to_border_radius(5.0))
                    .create_effect(move |cx, ent| {
                        let is_pressed = pressed.get(cx);
                        let is_hovering = hovering.get(cx);
                        let base_color = match variant.get(cx) {
                            ButtonVariant::Default => colors::U3,
                            ButtonVariant::Primary => colors::PRIMARY,
                            ButtonVariant::Danger => colors::DESTRUCTIVE,
                            ButtonVariant::Selected => colors::U4,
                        };
                        let color = match (is_pressed, is_hovering) {
                            (true, _) => base_color.lighter(0.05),
                            (false, true) => base_color.lighter(0.01),
                            (false, false) => base_color,
                        };
                        let mut bg = cx.world_mut().get_mut::<BackgroundColor>(ent).unwrap();
                        bg.0 = color.into();
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
                    }),
                self.children.clone(),
            ))
    }
}
