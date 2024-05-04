use bevy::{prelude::*, ui};
use bevy_reactor::*;
use obsidian_ui::{colors, controls::Spacer, typography};

use crate::InspectableField;

fn style_inspector_field_label(ss: &mut StyleBuilder) {
    ss.display(ui::Display::Flex)
        .flex_direction(ui::FlexDirection::Row)
        .align_items(ui::AlignItems::Center)
        .justify_content(ui::JustifyContent::FlexStart)
        .font_size(16)
        .color(colors::DIM);
}

/// Label for editable struct field in an inspector.
pub struct FieldLabel {
    /// The content of the label.
    pub field: InspectableField,
}

impl ViewTemplate for FieldLabel {
    fn create(&self, _cx: &mut Cx) -> impl Into<ViewRef> {
        Element::<NodeBundle>::new()
            .with_styles((typography::text_default, style_inspector_field_label))
            .with_children((self.field.name(), Spacer))
    }
}
