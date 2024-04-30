use crate::{colors, typography};
use bevy::{prelude::*, ui};
use bevy_reactor::*;

fn style_inspector_group(ss: &mut StyleBuilder) {
    ss.display(ui::Display::Flex)
        .flex_direction(ui::FlexDirection::Column)
        .align_items(ui::AlignItems::Stretch);
}

fn style_inspector_group_header(ss: &mut StyleBuilder) {
    ss.display(ui::Display::Flex)
        .flex_direction(ui::FlexDirection::Row)
        .align_items(ui::AlignItems::Center)
        .justify_content(ui::JustifyContent::SpaceBetween)
        .height(24)
        .font_size(16)
        .background_color(colors::U3)
        .border_radius(ui::BorderRadius {
            top_left: ui::Val::Px(4.0),
            top_right: ui::Val::Px(4.0),
            bottom_left: ui::Val::Px(0.0),
            bottom_right: ui::Val::Px(0.0),
        })
        .color(colors::FOREGROUND)
        .padding((8, 0));
}

fn style_inspector_group_body(ss: &mut StyleBuilder) {
    ss.display(ui::Display::Flex)
        .flex_direction(ui::FlexDirection::Column)
        .align_items(ui::AlignItems::Stretch)
        .border_color(colors::U3)
        .border(ui::UiRect {
            left: ui::Val::Px(1.0),
            right: ui::Val::Px(1.0),
            top: ui::Val::Px(0.0),
            bottom: ui::Val::Px(1.0),
        })
        .border_left(1)
        .border_right(1)
        .border_bottom(1)
        .border_radius(ui::BorderRadius {
            top_left: ui::Val::Px(0.0),
            top_right: ui::Val::Px(0.0),
            bottom_left: ui::Val::Px(4.0),
            bottom_right: ui::Val::Px(4.0),
        })
        .padding((8, 4));
}

/// Displays a inspector group card with a title and a body.
#[derive(Clone, Default)]
pub struct InspectorGroup {
    /// The content of the title section.
    pub title: ViewRef,
    /// The content of the body section.
    pub body: ViewRef,
}

impl ViewTemplate for InspectorGroup {
    fn create(&self, _cx: &mut Cx) -> impl Into<ViewRef> {
        Element::<NodeBundle>::new()
            .with_styles(style_inspector_group)
            .with_children((
                Element::<NodeBundle>::new()
                    .with_styles((typography::text_default, style_inspector_group_header))
                    .with_children(self.title.clone()),
                Element::<NodeBundle>::new()
                    .with_styles(style_inspector_group_body)
                    .with_children(self.body.clone()),
            ))
    }
}
