use std::sync::Arc;

use crate::{
    colors, hover_signal::CreateHoverSignal, prelude::RoundedCorners, size::Size, typography,
};
use bevy::{
    a11y::{
        accesskit::{NodeBuilder, Role},
        AccessibilityNode,
    },
    color::Luminance,
    prelude::*,
    ui,
};
use bevy_mod_stylebuilder::*;
use bevy_reactor_signals::{Callback, Cx, IntoSignal, RunContextSetup, Signal};
use bevy_reactor_views::*;

/// The variant determines the button's color scheme
#[derive(Clone, Copy, PartialEq, Default, Debug)]
pub enum ButtonVariant {
    /// The default button apperance.
    #[default]
    Default,

    /// A more prominent, "call to action", appearance.
    Primary,

    /// An appearance indicating a potentially dangerous action.
    Danger,

    /// A button that is in a "toggled" state.
    Selected,
}

pub(crate) fn style_button(ss: &mut StyleBuilder) {
    ss.border(1)
        .display(ui::Display::Flex)
        .flex_direction(ui::FlexDirection::Row)
        .justify_content(ui::JustifyContent::Center)
        .align_items(ui::AlignItems::Center)
        .align_content(ui::AlignContent::Center)
        .padding((12, 0))
        .border(0)
        .color(colors::FOREGROUND);
    // .cursor(CursorIcon::System(SystemCursorIcon::Pointer));
}

pub(crate) fn style_button_bg(ss: &mut StyleBuilder) {
    ss.display(ui::Display::Grid)
        .position(ui::PositionType::Absolute)
        .left(0)
        .right(0)
        .top(0)
        .bottom(0);
}

/// Button widget
pub struct Button {
    /// Color variant - default, primary or danger.
    pub variant: Signal<ButtonVariant>,

    /// Button size.
    pub size: Size,

    /// Whether the button is disabled.
    pub disabled: Signal<bool>,

    /// The content to display inside the button.
    pub children: Arc<dyn View>,

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

    /// If true, render the button in a 'minimal' style with no background and reduced padding.
    pub minimal: bool,
}

impl Button {
    /// Construct a new `Button`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the button color variant.
    pub fn variant(mut self, variant: impl IntoSignal<ButtonVariant>) -> Self {
        self.variant = variant.into_signal();
        self
    }

    /// Method which switches between `default` and `selected` style variants based on a boolean.
    /// Often used for toggle buttons or toolbar items.
    pub fn selected(mut self, selected: bool) -> Self {
        self.variant = if selected {
            ButtonVariant::Selected
        } else {
            ButtonVariant::Default
        }
        .into_signal();
        self
    }

    /// Set whether to render the button in a 'minimal' style with no background and reduced padding.
    pub fn minimal(mut self, minimal: bool) -> Self {
        self.minimal = minimal;
        self
    }

    /// Set the button size.
    pub fn size(mut self, size: Size) -> Self {
        self.size = size;
        self
    }

    /// Set the button disabled state.
    pub fn disabled(mut self, disabled: impl IntoSignal<bool>) -> Self {
        self.disabled = disabled.into_signal();
        self
    }

    /// Set the child views for this element.
    pub fn children<V: IntoView>(mut self, children: V) -> Self {
        self.children = children.into_view();
        self
    }

    /// Set the additional styles for the button.
    pub fn style<S: StyleTuple + 'static>(mut self, style: S) -> Self {
        self.style = style.into_handle();
        self
    }

    /// Set callback when clicked
    pub fn on_click(mut self, callback: Callback) -> Self {
        self.on_click = Some(callback);
        self
    }

    /// Set the tab index of the button.
    pub fn tab_index(mut self, tab_index: i32) -> Self {
        self.tab_index = tab_index;
        self
    }

    /// Set which corners to render rounded.
    pub fn corners(mut self, corners: RoundedCorners) -> Self {
        self.corners = corners;
        self
    }

    /// Set whether to autofocus the button when it's added to the UI.
    pub fn autofocus(mut self, autofocus: bool) -> Self {
        self.autofocus = autofocus;
        self
    }
}

impl Default for Button {
    fn default() -> Self {
        Self {
            variant: Signal::default(),
            size: Size::default(),
            disabled: default(),
            children: Arc::new(()),
            style: StyleHandle::none(),
            on_click: None,
            tab_index: 0,
            corners: RoundedCorners::default(),
            autofocus: false,
            minimal: false,
        }
    }
}

impl ViewTemplate for Button {
    fn create(&self, cx: &mut Cx) -> impl IntoView {
        let id = cx.create_entity();
        let variant = self.variant;
        let pressed = cx.create_mutable::<bool>(false);
        let hovering = cx.create_hover_signal(id);
        // let focused = cx.create_focus_visible_signal(id);
        let focused = Signal::Constant(false);

        let disabled = self.disabled;
        let corners = self.corners;
        let minimal = self.minimal;

        let size = self.size;

        Element::<NodeBundle>::for_entity(id)
            .named("Button")
            .style((
                typography::text_default,
                style_button,
                move |ss: &mut StyleBuilder| {
                    ss.min_height(size.height()).font_size(size.font_size());
                    if minimal {
                        ss.padding(0);
                    } else {
                        ss.padding((size.font_size() * 0.75, 0));
                    }
                },
                self.style.clone(),
            ))
            .insert((
                // TabIndex(self.tab_index),
                AccessibilityNode::from(NodeBuilder::new(Role::Button)),
                // {
                //     let on_click = self.on_click;
                //     On::<Pointer<Click>>::run(move |world: &mut World| {
                //         let mut focus = world.get_resource_mut::<Focus>().unwrap();
                //         focus.0 = Some(id);
                //         if !disabled.get(world) {
                //             let mut event = world
                //                 .get_resource_mut::<ListenerInput<Pointer<Click>>>()
                //                 .unwrap();
                //             event.stop_propagation();
                //             if let Some(on_click) = on_click {
                //                 world.run_callback(on_click, ());
                //             }
                //         }
                //     })
                // },
                // On::<Pointer<DragStart>>::run(move |world: &mut World| {
                //     if !disabled.get(world) {
                //         pressed.set(world, true);
                //     }
                // }),
                // On::<Pointer<DragEnd>>::run(move |world: &mut World| {
                //     if !disabled.get(world) {
                //         pressed.set(world, false);
                //     }
                // }),
                // On::<Pointer<DragEnter>>::run(move |world: &mut World| {
                //     if !disabled.get(world) {
                //         pressed.set(world, true);
                //     }
                // }),
                // On::<Pointer<DragLeave>>::run(move |world: &mut World| {
                //     if !disabled.get(world) {
                //         pressed.set(world, false);
                //     }
                // }),
                // On::<Pointer<PointerCancel>>::run(move |world: &mut World| {
                //     println!("PointerCancel");
                //     if !disabled.get(world) {
                //         pressed.set(world, false);
                //     }
                // }),
                // On::<KeyPressEvent>::run({
                //     let on_click = self.on_click;
                //     move |world: &mut World| {
                //         if !disabled.get(world) {
                //             let mut event = world
                //                 .get_resource_mut::<ListenerInput<KeyPressEvent>>()
                //                 .unwrap();
                //             if !event.repeat
                //                 && (event.key_code == KeyCode::Enter
                //                     || event.key_code == KeyCode::Space)
                //             {
                //                 event.stop_propagation();
                //                 if let Some(on_click) = on_click {
                //                     world.run_callback(on_click, ());
                //                 }
                //             }
                //         }
                //     }
                // }),
            ))
            // .insert_if(self.autofocus, AutoFocus)
            .observe(|trigger: Trigger<Pointer<Click>>| {
                println!("Click: {:?}", trigger);
            })
            .children((
                Element::<NodeBundle>::new()
                    .named("Button::Background")
                    .style(style_button_bg)
                    .insert(corners.to_border_radius(self.size.border_radius()))
                    .style_dyn(
                        move |cx| {
                            if minimal {
                                colors::TRANSPARENT
                            } else {
                                button_bg_color(
                                    variant.get(cx),
                                    disabled.get(cx),
                                    pressed.get(cx),
                                    hovering.get(cx),
                                )
                            }
                        },
                        |color, sb| {
                            sb.background_color(color);
                        },
                    )
                    .style_dyn(
                        move |cx| focused.get(cx),
                        |is_focused, sb| {
                            if is_focused {
                                sb.outline_color(colors::FOCUS)
                                    .outline_width(2)
                                    .outline_offset(2);
                            } else {
                                sb.outline_color(colors::TRANSPARENT)
                                    .outline_width(0)
                                    .outline_offset(0);
                            }
                        },
                    ),
                self.children.clone(),
            ))
    }
}

pub(crate) fn button_bg_color(
    variant: ButtonVariant,
    is_disabled: bool,
    is_pressed: bool,
    is_hovering: bool,
) -> Srgba {
    let base_color = match variant {
        ButtonVariant::Default => colors::U3,
        ButtonVariant::Primary => colors::PRIMARY,
        ButtonVariant::Danger => colors::DESTRUCTIVE,
        ButtonVariant::Selected => colors::U4,
    };
    match (is_disabled, is_pressed, is_hovering) {
        (true, _, _) => base_color.with_alpha(0.2),
        (_, true, _) => base_color.lighter(0.05),
        (_, false, true) => base_color.lighter(0.02),
        (_, false, false) => base_color,
    }
}
