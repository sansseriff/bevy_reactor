use super::Icon;
use crate::{
    animation::{AnimatedRotation, AnimatedTransition},
    colors,
    cursor::StyleBuilderCursor,
    focus::{KeyPressEvent, TabIndex},
    hooks::CreateFocusSignal,
    size::Size,
};
use bevy::{
    a11y::{
        accesskit::{NodeBuilder, Role},
        AccessibilityNode, Focus,
    },
    prelude::*,
    ui,
};
use bevy_mod_picking::prelude::*;
use bevy_reactor::*;

fn style_toggle(ss: &mut StyleBuilder) {
    ss.display(ui::Display::Flex)
        .flex_direction(ui::FlexDirection::Row)
        .justify_content(ui::JustifyContent::Center)
        .align_items(ui::AlignItems::Center)
        .align_content(ui::AlignContent::Center)
        .color(colors::FOREGROUND)
        .cursor(CursorIcon::Pointer);
}

/// A widget which displays small toggleable chevron that can be used to control whether
/// a panel is visible or hidden.
#[derive(Default)]
pub struct DisclosureToggle {
    /// Whether the toggle is in an expanded state.
    pub expanded: Signal<bool>,

    /// Button size.
    pub size: Size,

    /// Whether the button is disabled.
    pub disabled: Signal<bool>,

    /// Additional styles to be applied to the button.
    pub style: StyleHandle,

    /// Callback called when the state is toggled
    pub on_change: Option<Callback<bool>>,

    /// The tab index of the button (default 0).
    pub tab_index: i32,

    /// If true, set focus to this button when it's added to the UI.
    pub autofocus: bool,
}

impl DisclosureToggle {
    /// Construct a new `DisclosureToggle`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the expanded state of the button.
    pub fn expanded(mut self, expanded: impl IntoSignal<bool>) -> Self {
        self.expanded = expanded.into_signal();
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

    /// Set the additional styles for the button.
    pub fn style<S: StyleTuple + 'static>(mut self, style: S) -> Self {
        self.style = style.into_handle();
        self
    }

    /// Set callback when clicked
    pub fn on_change(mut self, callback: Callback<bool>) -> Self {
        self.on_change = Some(callback);
        self
    }

    /// Set the tab index of the button.
    pub fn tab_index(mut self, tab_index: i32) -> Self {
        self.tab_index = tab_index;
        self
    }

    /// Set whether to autofocus the button when it's added to the UI.
    pub fn autofocus(mut self, autofocus: bool) -> Self {
        self.autofocus = autofocus;
        self
    }
}

impl ViewTemplate for DisclosureToggle {
    fn create(&self, cx: &mut Cx) -> impl IntoView {
        let disabled = self.disabled;
        let checked = self.expanded;
        // let on_change = self.on_change;
        let id = cx.create_entity();
        let hovering = cx.create_hover_signal(id);
        let focused = cx.create_focus_visible_signal(id);

        Element::<NodeBundle>::for_entity(id)
            .named("DisclosureToggle")
            .style((style_toggle, self.style.clone()))
            .insert((
                TabIndex(self.tab_index),
                AccessibilityNode::from(NodeBuilder::new(Role::CheckBox)),
                {
                    let on_change = self.on_change;
                    On::<Pointer<Click>>::run(move |world: &mut World| {
                        let mut focus = world.get_resource_mut::<Focus>().unwrap();
                        focus.0 = Some(id);
                        if !disabled.get(world) {
                            let next_checked = checked.get(world);
                            if let Some(on_click) = on_change {
                                world.run_callback(on_click, !next_checked);
                            }
                        }
                    })
                },
                On::<KeyPressEvent>::run({
                    let on_change = self.on_change;
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
                                let next_checked = checked.get(world);
                                if let Some(on_click) = on_change {
                                    world.run_callback(on_click, !next_checked);
                                }
                            }
                        }
                    }
                }),
            ))
            .create_effect(move |cx, en| {
                let checked = checked.get(cx);
                let mut entt = cx.world_mut().entity_mut(en);
                let angle = if checked {
                    std::f32::consts::PI * 0.5
                } else {
                    0.
                };
                let target = Quat::from_rotation_z(angle);
                AnimatedTransition::<AnimatedRotation>::start(&mut entt, target, 0.3);
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
            .children(
                Icon::new("obsidian_ui://icons/chevron_right.png")
                    .color(cx.create_derived(move |cx| {
                        let is_disabled = disabled.get(cx);
                        let is_hover = hovering.get(cx);
                        match (is_disabled, is_hover) {
                            (true, _) => Color::from(colors::DIM).with_alpha(0.2),
                            (false, true) => Color::from(colors::FOREGROUND),
                            (false, false) => Color::from(colors::DIM),
                        }
                    }))
                    .size(match self.size {
                        Size::Xl => Vec2::splat(24.),
                        Size::Lg => Vec2::splat(20.),
                        Size::Md => Vec2::splat(18.),
                        Size::Sm => Vec2::splat(16.),
                        Size::Xs => Vec2::splat(13.),
                        Size::Xxs => Vec2::splat(12.),
                        Size::Xxxs => Vec2::splat(11.),
                    })
                    .style(|ss: &mut StyleBuilder| {
                        ss.margin((4, 0));
                    }),
            )
    }
}
