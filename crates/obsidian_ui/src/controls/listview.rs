use bevy::{prelude::*, ui};
use bevy_reactor::*;

use crate::colors;

use super::ScrollView;

fn style_listview(ss: &mut StyleBuilder) {
    ss.background_color(colors::U1)
        .border_radius(5.0)
        .padding(4);
}

fn style_listview_inner(ss: &mut StyleBuilder) {
    ss.display(ui::Display::Flex)
        .flex_direction(ui::FlexDirection::Column)
        .align_items(ui::AlignItems::Stretch);
}

/// A scrollable list of items.
#[derive(Clone, Default)]
pub struct ListView {
    /// Additional styles to be applied to the list view.
    pub style: StyleHandle,

    /// The content of the dialog header.
    pub children: ViewRef,
}

impl ViewTemplate for ListView {
    fn create(&self, _cx: &mut Cx) -> impl Into<ViewRef> {
        ScrollView {
            children: ViewRef::new(
                Element::<NodeBundle>::new()
                    .named("ListView")
                    .with_styles(style_listview_inner)
                    .with_children(self.children.clone()),
            ),
            style: StyleHandle::new((style_listview, self.style.clone())),
            scroll_enable_y: true,
            ..default()
        }
    }
}
