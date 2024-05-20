use bevy::{prelude::*, ui};
use bevy_reactor::*;
use bevy_reactor_signals::{Cx, RunContextSetup};
use obsidian_ui::{
    colors,
    controls::{IconButton, Spacer},
    size::Size,
    typography,
};

use crate::InspectableField;

fn style_field_label(ss: &mut StyleBuilder) {
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
    fn create(&self, cx: &mut Cx) -> impl IntoView {
        let can_remove = self.field.can_remove();
        let field = self.field.clone();
        let remove = cx.create_callback(move |cx, _| {
            field.remove(cx);
        });
        Element::<NodeBundle>::new()
            .style((typography::text_default, style_field_label))
            .children((
                self.field.name(),
                Cond::new(
                    move |_cx| can_remove,
                    move || {
                        IconButton::new("obsidian_ui://icons/close.png")
                            .size(Size::Xs)
                            .minimal(true)
                            .on_click(remove)
                    },
                    || (),
                ),
                Spacer,
            ))
    }
}

fn style_field_label_wide(ss: &mut StyleBuilder) {
    ss.display(ui::Display::Flex)
        .flex_direction(ui::FlexDirection::Row)
        .align_items(ui::AlignItems::Center)
        .justify_content(ui::JustifyContent::FlexStart)
        .grid_column_span(2)
        .font_size(16)
        .min_width(64)
        .color(colors::DIM);
}

/// Label for editable struct field in an inspector.
pub struct FieldLabelWide {
    /// The content of the label.
    pub field: InspectableField,
    /// Name to display.
    pub name: Option<ViewRef>,
    /// Additional buttons in the label.
    pub buttons: Option<ViewRef>,
}

impl FieldLabelWide {
    /// Create a new field label.
    pub fn new(field: InspectableField) -> Self {
        Self {
            field,
            name: None,
            buttons: None,
        }
    }

    /// Set the name of the field.
    pub fn name(mut self, name: impl IntoView) -> Self {
        self.name = Some(name.into_view());
        self
    }

    /// Set additional buttons in the label.
    pub fn buttons(mut self, buttons: impl IntoView) -> Self {
        self.buttons = Some(buttons.into_view());
        self
    }
}

impl ViewTemplate for FieldLabelWide {
    fn create(&self, cx: &mut Cx) -> impl IntoView {
        let can_remove = self.field.can_remove();
        let field = self.field.clone();
        let remove = cx.create_callback(move |cx, _| {
            field.remove(cx);
        });
        Element::<NodeBundle>::new()
            .style((typography::text_default, style_field_label_wide))
            .children((
                // TODO: Disclosure
                self.name.clone(),
                // Optional item count
                Spacer,
                self.buttons.clone(),
                Cond::new(
                    move |_cx| can_remove,
                    move || {
                        IconButton::new("obsidian_ui://icons/close.png")
                            .size(Size::Xs)
                            .minimal(true)
                            .on_click(remove)
                    },
                    || (),
                ),
            ))
    }
}
