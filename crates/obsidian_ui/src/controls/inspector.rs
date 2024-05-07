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
    pub title: ChildArray,
    /// The content of the body section.
    pub body: ChildArray,
    /// Whether the group is expanded or not. When collapsed, only the title is shown.
    pub expanded: Signal<bool>,
}

impl InspectorGroup {
    /// Create a new inspector group with the given title and body.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the title of the inspector group.
    pub fn title<V: ChildViewTuple>(mut self, title: V) -> Self {
        self.title = title.to_child_array();
        self
    }

    /// Set the body of the inspector group.
    pub fn body<V: ChildViewTuple>(mut self, body: V) -> Self {
        self.body = body.to_child_array();
        self
    }

    /// Set the expanded signal of the inspector group.
    pub fn expanded(mut self, expanded: impl IntoSignal<bool>) -> Self {
        self.expanded = expanded.into_signal();
        self
    }
}

impl ViewTemplate for InspectorGroup {
    fn create(&self, _cx: &mut Cx) -> impl IntoView {
        let expanded = self.expanded;
        let body = self.body.clone();
        Element::<NodeBundle>::new()
            .style(style_inspector_group)
            .children((
                Element::<NodeBundle>::new()
                    .style((typography::text_default, style_inspector_group_header))
                    .children(self.title.clone()),
                Cond::new(
                    move |cx| expanded.get(cx),
                    move || {
                        Element::<NodeBundle>::new()
                            .style(style_inspector_group_body)
                            .children(body.clone())
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
    fn create(&self, _cx: &mut Cx) -> impl IntoView {
        Element::<NodeBundle>::new()
            .style((
                typography::text_default,
                style_inspector_field_label,
                self.style.clone(),
            ))
            .child(&self.children)
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
    pub children: ChildArray,
    /// Additional styles for the label.
    pub style: StyleHandle,
}

impl InspectorFieldReadonlyValue {
    /// Create a new readonly value with the given text.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the child views for this element.
    pub fn children<V: ChildViewTuple>(mut self, children: V) -> Self {
        self.children = children.to_child_array();
        self
    }

    /// Set the additional styles for the button.
    pub fn style<S: StyleTuple + 'static>(mut self, style: S) -> Self {
        self.style = style.into_handle();
        self
    }
}

impl ViewTemplate for InspectorFieldReadonlyValue {
    fn create(&self, _cx: &mut Cx) -> impl IntoView {
        Element::<NodeBundle>::new()
            .style((
                typography::text_default,
                style_inspector_field_readonly_value,
                self.style.clone(),
            ))
            .children(self.children.clone())
    }
}
