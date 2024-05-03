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
        .padding_left(8)
        .padding_right(3);
}

fn style_inspector_group_body(ss: &mut StyleBuilder) {
    ss.display(ui::Display::Grid)
        .grid_auto_flow(ui::GridAutoFlow::Row)
        .grid_template_columns(vec![
            ui::RepeatedGridTrack::auto(1),
            ui::RepeatedGridTrack::flex(1, 1.),
        ])
        .column_gap(4)
        .row_gap(2)
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
        .padding_left(6)
        .padding_right(4)
        .padding_top(4)
        .padding_bottom(4);
}

/// Displays a inspector group card with a title and a body.
#[derive(Clone, Default)]
pub struct InspectorGroup {
    /// The content of the title section.
    pub title: ViewRef,
    /// The content of the body section.
    pub body: ViewRef,
    /// Whether the group is expanded or not. When collapsed, only the title is shown.
    pub expanded: Signal<bool>,
}

impl ViewTemplate for InspectorGroup {
    fn create(&self, _cx: &mut Cx) -> impl Into<ViewRef> {
        let expanded = self.expanded;
        let body = self.body.clone();
        Element::<NodeBundle>::new()
            .with_styles(style_inspector_group)
            .with_children((
                Element::<NodeBundle>::new()
                    .with_styles((typography::text_default, style_inspector_group_header))
                    .with_children(self.title.clone()),
                Cond::new(
                    move |cx| expanded.get(cx),
                    move || {
                        Element::<NodeBundle>::new()
                            .with_styles(style_inspector_group_body)
                            .with_children(body.clone())
                    },
                    || (),
                ),
            ))
    }
}

fn style_inspector_field_label(ss: &mut StyleBuilder) {
    ss.display(ui::Display::Flex)
        .flex_direction(ui::FlexDirection::Row)
        .align_items(ui::AlignItems::Center)
        .justify_content(ui::JustifyContent::FlexStart)
        .font_size(16)
        .color(colors::DIM);
}

/// Label for editable field in an inspector.
#[derive(Clone, Default)]
pub struct InspectorFieldLabel {
    /// The content of the label.
    pub children: ViewRef,
    /// Additional styles for the label.
    pub style: StyleHandle,
}

impl ViewTemplate for InspectorFieldLabel {
    fn create(&self, _cx: &mut Cx) -> impl Into<ViewRef> {
        Element::<NodeBundle>::new()
            .with_styles((
                typography::text_default,
                style_inspector_field_label,
                self.style.clone(),
            ))
            .with_child(&self.children)
    }
}

fn style_inspector_field_readonly_value(ss: &mut StyleBuilder) {
    ss.display(ui::Display::Flex)
        .flex_direction(ui::FlexDirection::Row)
        .align_items(ui::AlignItems::Center)
        .justify_content(ui::JustifyContent::FlexStart)
        .border(1)
        .border_color(colors::U3)
        .font_size(16)
        .color(colors::DIM)
        .padding((4, 1));
}

/// Readonly value displayed as text in the inspector.
#[derive(Clone, Default)]
pub struct InspectorFieldReadonlyValue {
    /// The text representation of the value.
    pub children: ViewRef,
    /// Additional styles for the label.
    pub style: StyleHandle,
}

impl ViewTemplate for InspectorFieldReadonlyValue {
    fn create(&self, _cx: &mut Cx) -> impl Into<ViewRef> {
        Element::<NodeBundle>::new()
            .with_styles((
                typography::text_default,
                style_inspector_field_readonly_value,
                self.style.clone(),
            ))
            .with_child(&self.children)
    }
}
