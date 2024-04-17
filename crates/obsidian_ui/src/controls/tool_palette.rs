use crate::{size::Size, RoundedCorners};
use bevy::{
    a11y::{
        accesskit::{NodeBuilder, Role},
        AccessibilityNode,
    },
    prelude::*,
    ui,
};
use bevy_reactor::*;

use super::{Button, ButtonVariant};

fn style_tool_palette(ss: &mut StyleBuilder) {
    ss.display(ui::Display::Grid)
        .gap(1)
        // .justify_content(ui::JustifyContent::Center)
        // .align_items(ui::AlignItems::Center)
        // .align_content(ui::AlignContent::Center);
        .grid_auto_rows(vec![ui::GridTrack::default()]);
}

#[derive(Clone, Debug, Default, Component)]
struct ToolPaletteContext {
    size: Size,
}

/// ToolPalette - a grid of tool buttons
#[derive(Default)]
pub struct ToolPalette {
    /// Button size.
    pub size: Size,

    /// The buttons to display.
    pub children: ViewRef,

    /// Additional styles to be applied to the palette.
    pub style: StyleHandle,

    /// Number of button columns
    pub columns: u16,
}

impl ViewTemplate for ToolPalette {
    fn create(&self, cx: &mut Cx) -> impl Into<ViewRef> {
        let columns = self.columns;

        cx.insert(ToolPaletteContext { size: self.size });

        Element::<NodeBundle>::new()
            .named("ToolPalette")
            .with_styles((
                style_tool_palette,
                move |ss: &mut StyleBuilder| {
                    ss.grid_template_columns(vec![ui::RepeatedGridTrack::auto(columns)]);
                },
                self.style.clone(),
            ))
            .insert(AccessibilityNode::from(NodeBuilder::new(Role::Group)))
            .with_children(self.children.clone())
    }
}

/// A button in a ToolPalette.
pub struct ToolButton {
    /// Color variant - default, primary or danger.
    pub variant: Signal<ButtonVariant>,

    /// Whether the button is disabled.
    pub disabled: Signal<bool>,

    /// The content to display inside the button.
    pub children: ViewRef,

    /// Callback called when clicked
    pub on_click: Option<Callback>,

    /// The tab index of the button (default 0).
    pub tab_index: i32,

    /// Which corners to render rounded.
    pub corners: RoundedCorners,

    /// If true, set focus to this button when it's added to the UI.
    pub autofocus: bool,
}

impl Default for ToolButton {
    fn default() -> Self {
        Self {
            variant: Default::default(),
            disabled: Default::default(),
            children: Default::default(),
            on_click: Default::default(),
            tab_index: 0,
            corners: RoundedCorners::None,
            autofocus: false,
        }
    }
}

impl ViewTemplate for ToolButton {
    fn create(&self, cx: &mut Cx) -> impl Into<ViewRef> {
        let context = cx.use_inherited_component::<ToolPaletteContext>().unwrap();
        Button {
            size: context.size,
            variant: self.variant,
            disabled: self.disabled,
            children: self.children.clone(),
            on_click: self.on_click,
            tab_index: self.tab_index,
            autofocus: self.autofocus,
            corners: self.corners,
            // style: StyleHandle::new(move |ss: &mut StyleBuilder| {
            //     println!("Index: {:?}", ss.child_index());
            //     // ss.grid_template_columns(vec![ui::RepeatedGridTrack::auto(columns)]);
            // }),
            // corners: self.corners,
            ..default()
        }
    }
}
